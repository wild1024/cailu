//! 认证处理器：登录、登出、令牌刷新与当前用户信息。
//!
//! 处理器承担**业务编排**（参数校验、分支判定、响应组装）与框架快捷方法的直接
//! 调用（按用户名 / 主键查用户、`of(...).set(...).update` 解锁等）；需要编写 SQL
//! 的查询与领域逻辑集中在 [`crate::system::auth::model`]。日志一律通过
//! [`crate::support::log`] 的 `record_*` 函数以**非阻塞**方式投递，不拖慢主流程。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::net::SocketAddr;

use crate::shared::constants::{self, EnableStatus};
use crate::shared::error_code;
use crate::support::log::dto::LogPageQuery;
use crate::support::log::model::{JsonLogTable, page_json_logs};
use crate::support::log::{
    self, AuditLogInput, AuthLogInput, AuthStatus, AuthType, PasswordChangeLogInput,
    SecurityLogInput,
};
use crate::support::online::model::{OnlineSession, OnlineSessionModel};
use crate::system::auth::dto::{
    ChangePasswordReq, CurrentUserRes, LoginReq, LoginUserRes, RefreshTokenReq, TokenRes,
    UpdateProfileReq,
};
use crate::system::auth::model;
use crate::system::menu::dto::MenuRes;
use crate::system::user::model::SysUserModel;
use crate::utils;

/// 令牌类型（固定 `Bearer`）。
const TOKEN_TYPE: &str = "Bearer";

// ==================== 接口 ====================

/// 账密登录。
///
/// # 参数
///
/// - `headers`：请求头（用于派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP 做限流）。
/// - `req`：登录请求体（用户名 / 密码）。
///
/// # 返回值
///
/// 成功返回访问令牌、刷新令牌与登录用户摘要。
///
/// # 异常情况
///
/// - 429：登录尝试过于频繁；
/// - 401：用户名或密码错误；
/// - 403：账号已禁用或被锁定；
/// - 500：内部错误。
///
/// # 日志
///
/// 成功写 `auth_logs`（`LOGIN/SUCCESS`）与 `audit_logs`（`LOGIN_SUCCESS`）；
/// 失败写 `auth_logs`（`LOGIN/FAIL`）与 `security_logs`（`LOGIN_FAIL`）。
/// 锁定阈值与解锁时长取自 `sys_config`（缺省回退常量）。日志以非阻塞方式投递，
/// 不参与登录耗时。
#[alun::no_auth("/system/auth/login")]
#[alun::post("/system/auth/login")]
pub async fn login(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    ValidatedJson(req): ValidatedJson<LoginReq>,
) -> ResResult<TokenRes> {
    // 账号密码统一去除首尾空白后再参与查库与校验：
    // 登录页已做同样处理，这里兜底直连 API / 其他客户端（如脚本、第三方系统）的场景。
    let req = LoginReq {
        username: req.username.trim().to_string(),
        password: req.password.trim().to_string(),
    };
    // 校验器只保证原始长度 >= 1，纯空白串 trim 后为空，需在此拦下
    if req.username.is_empty() || req.password.is_empty() {
        return Err(ApiError::new(
            400,
            error_code::PARAM_INVALID,
            "用户名或密码不能为空",
        ));
    }

    let mut timer = LoginTimer::new();
    let ip_text = utils::client_ip(&headers, Some(&connect_info))
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    check_login_rate(&req.username, &ip_text).await?;
    timer.mark("rate_limit");

    let (max_fail, lock_minutes) = model::login_security_policy().await;
    timer.mark("security_policy");

    let user = match SysUserModel::find_first_by(db(), "username", (req.username.as_str(),)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            timer.mark("load_user");
            log_login_failure(
                &headers,
                &connect_info,
                &req.username,
                None,
                "用户不存在",
                &timer,
            );
            return Err(ApiError::new(
                401,
                error_code::AUTH_BAD_CREDENTIALS,
                "用户名或密码错误",
            ));
        }
        Err(e) => return Err(utils::db_error(e, "查询登录用户")),
    };

    // 锁定期内直接拒绝；锁定已到期则自动解锁（重置失败计数）
    if is_locked_now(&user, lock_minutes) {
        timer.mark("load_user");
        log_login_failure(
            &headers,
            &connect_info,
            &req.username,
            Some(user.id),
            "账号锁定中",
            &timer,
        );
        return Err(ApiError::new(
            403,
            error_code::AUTH_ACCOUNT_LOCKED,
            "账号已被锁定，请稍后再试",
        ));
    }
    if user.is_locked {
        SysUserModel::of(user.id)
            .set(SysUserModel::is_locked(), false)
            .set(SysUserModel::login_fail_count(), 0i16)
            .set(
                SysUserModel::lock_time(),
                None::<chrono::DateTime<chrono::Utc>>,
            )
            .update(db())
            .await
            .map(|_| ())
            .map_err(|e| utils::db_error(e, "解锁账号"))?;
    }

    if user.status != EnableStatus::enabled {
        timer.mark("load_user");
        log_login_failure(
            &headers,
            &connect_info,
            &req.username,
            Some(user.id),
            "账号已禁用",
            &timer,
        );
        return Err(ApiError::new(
            403,
            error_code::AUTH_ACCOUNT_DISABLED,
            "账号已被禁用，请联系管理员",
        ));
    }

    let password_ok = Crypto::verify_password(&req.password, &user.password).map_err(|e| {
        tracing::error!(error = %e, user_id = %user.id, "密码校验失败");
        ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
    })?;

    timer.mark("verify_password");

    if !password_ok {
        let fails = user.login_fail_count.saturating_add(1);
        let is_lock = fails >= max_fail;
        model::record_login_failure(&user.id, fails, is_lock, chrono::Utc::now())
            .await
            .map_err(|e| utils::db_error(e, "更新登录失败计数"))?;
        let reason = if is_lock {
            "密码错误，账号已锁定"
        } else {
            "密码错误"
        };
        timer.mark("record_failure");
        log_login_failure(
            &headers,
            &connect_info,
            &req.username,
            Some(user.id),
            reason,
            &timer,
        );
        return Err(ApiError::new(
            401,
            error_code::AUTH_BAD_CREDENTIALS,
            "用户名或密码错误",
        ));
    }

    // 登录成功：重置失败计数、清除锁定并记录最后登录时间
    timer.mark("reset_status");
    SysUserModel::of(user.id)
        .set(SysUserModel::login_fail_count(), 0i16)
        .set(SysUserModel::is_locked(), false)
        .set(
            SysUserModel::lock_time(),
            None::<chrono::DateTime<chrono::Utc>>,
        )
        .set(SysUserModel::last_login_at(), Some(chrono::Utc::now()))
        .update(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "更新登录状态"))?;

    timer.mark("reset_status_done");
    let (token, claims) = issue_token(&user, Some(&mut timer)).await?;
    register_online_session(&headers, &connect_info, &claims).await;
    timer.mark("online_session");

    log::record_auth(
        &headers,
        &connect_info,
        AuthLogInput {
            user_id: Some(user.id),
            username: Some(user.username.clone()),
            auth_type: AuthType::Login,
            auth_status: AuthStatus::Success,
            error_message: None,
            login_duration: Some(timer.total_ms()),
        },
    );
    timer.report();
    log::record_audit(
        &headers,
        &connect_info,
        AuditLogInput {
            user_id: Some(user.id),
            username: Some(user.username.clone()),
            operation: "LOGIN_SUCCESS".to_string(),
            target_type: Some("User".to_string()),
            target_id: Some(user.id.to_string()),
            before_data: None,
            after_data: None,
            detail: None,
        },
    );

    Ok(Res::ok(token))
}

/// 用刷新令牌换取新的令牌对。
///
/// 旧刷新令牌会被加入黑名单（一次性使用），权限等声明重新从数据库加载，
/// 不直接复用 `JWT::refresh`（后者会丢失角色 / 权限）。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `req`：刷新令牌请求体。
///
/// # 返回值
///
/// 成功返回新的访问令牌与刷新令牌。
///
/// # 异常情况
///
/// - 401：刷新令牌无效、类型错误或已被撤销；
/// - 403：账号已被禁用；
/// - 500：内部错误。
#[alun::no_auth("/system/auth/refresh")]
#[alun::post("/system/auth/refresh")]
pub async fn refresh(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    ValidatedJson(req): ValidatedJson<RefreshTokenReq>,
) -> ResResult<TokenRes> {
    let jwt = JWT::from_config();
    let claims = jwt.validate(&req.refresh_token).map_err(|_| {
        ApiError::new(
            401,
            error_code::AUTH_REFRESH_TOKEN_INVALID,
            "刷新令牌无效或已过期",
        )
    })?;

    if claims.token_type != Some(TokenType::Refresh) {
        return Err(ApiError::new(
            401,
            error_code::AUTH_REFRESH_TOKEN_INVALID,
            "令牌类型不正确",
        ));
    }

    if jwt.is_blacklisted(&claims).await {
        return Err(ApiError::new(
            401,
            error_code::AUTH_TOKEN_REVOKED,
            "刷新令牌已被撤销",
        ));
    }

    // 旧刷新令牌一次性作废
    jwt.blacklist(&claims).await;

    let user_id = utils::require_uuid(&claims.sub, "sub")?;
    let user = match SysUserModel::find_or_none(db(), &user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::record_auth(
                &headers,
                &connect_info,
                AuthLogInput {
                    user_id: Some(user_id),
                    username: claims.username.clone(),
                    auth_type: AuthType::RefreshToken,
                    auth_status: AuthStatus::Fail,
                    error_message: Some("刷新令牌对应的用户不存在".to_string()),
                    login_duration: None,
                },
            );
            return Err(ApiError::new(
                401,
                error_code::AUTH_REFRESH_TOKEN_INVALID,
                "刷新令牌对应的用户不存在",
            ));
        }
        Err(e) => return Err(utils::db_error(e, "查询刷新用户")),
    };

    if user.status != EnableStatus::enabled {
        log::record_auth(
            &headers,
            &connect_info,
            AuthLogInput {
                user_id: Some(user.id),
                username: Some(user.username.clone()),
                auth_type: AuthType::RefreshToken,
                auth_status: AuthStatus::Fail,
                error_message: Some("账号已禁用".to_string()),
                login_duration: None,
            },
        );
        return Err(ApiError::new(
            403,
            error_code::AUTH_ACCOUNT_DISABLED,
            "账号已被禁用，请联系管理员",
        ));
    }

    // 刷新路径不采集分段耗时：令牌已存在，慢点只在首次登录需要定位。
    let (token, new_claims) = issue_token(&user, None).await?;
    register_online_session(&headers, &connect_info, &new_claims).await;

    log::record_auth(
        &headers,
        &connect_info,
        AuthLogInput {
            user_id: Some(user.id),
            username: Some(user.username.clone()),
            auth_type: AuthType::RefreshToken,
            auth_status: AuthStatus::Success,
            error_message: None,
            login_duration: None,
        },
    );
    Ok(Res::ok(token))
}

/// 登出：把当前访问令牌加入黑名单，并移除在线会话。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：认证中间件注入的令牌声明。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// 无（令牌缺失由认证中间件提前拦截为 401）。
///
/// # 日志
///
/// 写入 `auth_logs`（`LOGOUT/SUCCESS`）；写入失败不影响登出结果。
#[alun::post("/system/auth/logout")]
pub async fn logout(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    JWT::from_config().logout(&claims).await;
    revoke_online_session(&claims).await;
    log::record_auth(
        &headers,
        &connect_info,
        AuthLogInput {
            user_id: utils::current_user_uuid(&claims),
            username: claims.username.clone(),
            auth_type: AuthType::Logout,
            auth_status: AuthStatus::Success,
            error_message: None,
            login_duration: None,
        },
    );
    Ok(Res::ok_msg("已退出登录"))
}

/// 当前登录用户信息（含角色编码与权限码）。
///
/// # 参数
///
/// - `claims`：认证中间件注入的令牌声明。
///
/// # 返回值
///
/// 成功返回当前用户详情。
///
/// # 异常情况
///
/// - 401：令牌中的用户已不存在；
/// - 500：内部错误。
#[alun::get("/system/auth/me")]
pub async fn me(Extension(AuthClaims(claims)): Extension<AuthClaims>) -> ResResult<CurrentUserRes> {
    let user_id = utils::require_uuid(&claims.sub, "sub")?;
    let user = SysUserModel::find_or_none(db(), &user_id)
        .await
        .map_err(|e| utils::db_error(e, "查询当前用户"))?
        .ok_or_else(|| ApiError::new(401, error_code::USER_NOT_FOUND, "用户不存在"))?;

    let codes = model::load_role_codes(&user_id)
        .await
        .map_err(|e| utils::db_error(e, "查询用户角色"))?;
    let super_admin = model::is_super_admin(&codes);
    let permissions = model::load_permissions(&user_id, super_admin)
        .await
        .map_err(|e| utils::db_error(e, "查询用户权限"))?;

    Ok(Res::ok(CurrentUserRes {
        id: user.id.to_string(),
        username: user.username,
        real_name: user.real_name,
        email: user.email,
        mobile: user.mobile,
        dept_id: user.dept_id.map(|v| v.to_string()),
        roles: codes,
        permissions,
    }))
}

/// 当前登录用户的菜单树（仅目录与菜单，按 `sort_order` 升序）。
///
/// # 参数
///
/// - `claims`：认证中间件注入的令牌声明。
///
/// # 返回值
///
/// 成功返回菜单树根节点列表。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::get("/system/auth/menus")]
pub async fn menus(
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<Vec<MenuRes>> {
    let user_id = utils::require_uuid(&claims.sub, "sub")?;
    let super_admin = claims.is_super_admin();
    let nodes = model::load_user_menus(&user_id, super_admin)
        .await
        .map_err(|e| utils::db_error(e, "查询用户菜单"))?;
    Ok(Res::ok(build_menu_tree(nodes)))
}

/// 更新当前登录用户的个人资料（姓名 / 邮箱 / 手机号）。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：认证中间件注入的令牌声明。
/// - `req`：待更新的资料字段（`None` 表示保持原值，空串表示清空）。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：字段校验失败或邮箱格式非法；
/// - 401：令牌中的用户已不存在；
/// - 500：内部错误。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`PROFILE_UPDATE`）；日志写入失败不影响业务结果。
#[alun::put("/system/auth/profile")]
pub async fn update_profile(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<UpdateProfileReq>,
) -> ResResult<()> {
    let user_id = utils::require_uuid(&claims.sub, "sub")?;
    let mut user = SysUserModel::find_or_none(db(), &user_id)
        .await
        .map_err(|e| utils::db_error(e, "查询当前用户"))?
        .ok_or_else(|| ApiError::new(401, error_code::USER_NOT_FOUND, "用户不存在"))?;

    // 仅在提交了对应字段时覆盖，避免误清空其它资料
    if let Some(real_name) = req.real_name.as_deref() {
        user.real_name = utils::trimmed(Some(real_name)).map(str::to_string);
    }
    if let Some(email) = req.email.as_deref() {
        user.email = utils::trimmed(Some(email)).map(str::to_string);
    }
    if let Some(mobile) = req.mobile.as_deref() {
        user.mobile = utils::trimmed(Some(mobile)).map(str::to_string);
    }
    user.updated_at = chrono::Utc::now();
    user.update_by = utils::current_user_uuid(&claims);

    user.update(db())
        .await
        .map_err(|e| utils::db_error(e, "更新个人资料"))?;

    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "PROFILE_UPDATE",
            "User",
            user_id.to_string(),
            None,
            None,
        ),
    );
    Ok(Res::ok_msg("个人资料已更新"))
}

/// 修改当前登录用户的密码（需校验原密码）。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：认证中间件注入的令牌声明。
/// - `req`：原密码与新密码。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：新密码长度不足；
/// - 401：原密码错误，或令牌中的用户已不存在；
/// - 500：内部错误。
///
/// # 日志
///
/// 成功写 `password_change_logs`（`UPDATE/SELF/SUCCESS`）；原密码错误写
/// `password_change_logs`（`UPDATE/SELF/FAIL`）；日志写入失败不影响业务结果。
#[alun::put("/system/auth/password")]
pub async fn change_password(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<ChangePasswordReq>,
) -> ResResult<()> {
    let user_id = utils::require_uuid(&claims.sub, "sub")?;
    let user = SysUserModel::find_or_none(db(), &user_id)
        .await
        .map_err(|e| utils::db_error(e, "查询当前用户"))?
        .ok_or_else(|| ApiError::new(401, error_code::USER_NOT_FOUND, "用户不存在"))?;

    let password_ok = Crypto::verify_password(&req.old_password, &user.password).map_err(|e| {
        tracing::error!(error = %e, user_id = %user_id, "原密码校验失败");
        ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
    })?;
    if !password_ok {
        log::record_password_change(
            &headers,
            &connect_info,
            PasswordChangeLogInput {
                user_id,
                username: user.username.clone(),
                change_type: "UPDATE".to_string(),
                operator_type: "SELF".to_string(),
                operator_id: Some(user_id),
                change_status: "FAIL".to_string(),
                error_message: Some("原密码错误".to_string()),
            },
        );
        return Err(ApiError::new(
            401,
            error_code::AUTH_BAD_CREDENTIALS,
            "原密码错误",
        ));
    }

    let hashed = Crypto::hash_password(&req.new_password)
        .map_err(|_| ApiError::new(500, error_code::INTERNAL_ERROR, "密码加密失败"))?;
    SysUserModel::of(user_id)
        .set(SysUserModel::password(), hashed)
        .set(SysUserModel::updated_at(), chrono::Utc::now())
        .set(SysUserModel::update_by(), utils::current_user_uuid(&claims))
        .update(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "修改密码"))?;

    log::record_password_change(
        &headers,
        &connect_info,
        PasswordChangeLogInput {
            user_id,
            username: user.username.clone(),
            change_type: "UPDATE".to_string(),
            operator_type: "SELF".to_string(),
            operator_id: Some(user_id),
            change_status: "SUCCESS".to_string(),
            error_message: None,
        },
    );
    Ok(Res::ok_msg("密码已修改"))
}

/// 当前登录用户自己的登录（认证）日志分页。
///
/// 普通用户无需日志查看权限即可查看本人登录记录，因此按当前用户名校验，
/// 不额外声明权限码。
///
/// # 参数
///
/// - `claims`：认证中间件注入的令牌声明（取用户名做过滤）。
/// - `query`：分页参数（仅使用 `page` / `page_size`）。
///
/// # 返回值
///
/// 成功返回本人认证日志的 JSON 行分页（按时间倒序）。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::get("/system/auth/logs")]
pub async fn login_logs(
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    Query(query): Query<LogPageQuery>,
) -> ResResult<PageData<Vec<JsonValue>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);
    let result = page_json_logs(
        JsonLogTable::Auth,
        claims.username.clone(),
        None,
        None,
        page,
        page_size,
    )
    .await
    .map_err(|e| utils::db_error(e, "查询登录日志"))?;
    Ok(Res::page(
        result.list,
        result.total,
        result.page,
        result.size,
    ))
}

// ==================== 公共辅助 ====================

/// 登录耗时分段采集器。
///
/// 用于把「登录耗时偏长」拆到具体环节：调用方按执行顺序依次
/// [`LoginTimer::mark`]，结束时 `report()` 一次性输出各段毫秒数。只做计时与
/// 一条 `tracing::info!`，不参与业务判定、不写库。
struct LoginTimer {
    /// 分段名称与自上一段以来的耗时（毫秒）。
    steps: Vec<(&'static str, u128)>,
    /// 登录开始的时刻。
    start: std::time::Instant,
    /// 上一次打点的时刻。
    last: std::time::Instant,
}

impl LoginTimer {
    /// 创建采集器（起点即当前时刻）。
    fn new() -> Self {
        let start = std::time::Instant::now();
        Self {
            steps: Vec::new(),
            start,
            last: start,
        }
    }

    /// 记录自上一段（或起点）以来消耗的毫秒数，并把后续阶段命名。
    fn mark(&mut self, label: &'static str) -> u128 {
        let now = std::time::Instant::now();
        let spent = now.saturating_duration_since(self.last).as_millis();
        self.last = now;
        self.steps.push((label, spent));
        spent
    }

    /// 本次登录自起点到此刻的总耗时（毫秒）。
    fn total_ms(&self) -> i32 {
        std::time::Instant::now()
            .saturating_duration_since(self.start)
            .as_millis()
            .min(i32::MAX as u128) as i32
    }

    /// 输出分段耗时日志。
    fn report(&self) {
        let detail: Vec<String> = self
            .steps
            .iter()
            .map(|(label, ms)| format!("{label}={ms}ms"))
            .collect();
        tracing::info!(total = ?self.total_ms(), steps = ?detail, "登录耗时分解");
    }
}

/// 组装登录用户的角色编码、权限码与令牌对。
///
/// # 参数
///
/// - `user`：已通过身份校验的用户模型。
///
/// # 返回值
///
/// `(令牌响应, 访问令牌声明)`。声明用于登记在线会话（签发接口不直接返回 `jti`）。
///
/// # 异常情况
///
/// 角色 / 权限查询或令牌签发失败返回 500；签发后回读声明失败同样返回 500。
async fn issue_token(
    user: &SysUserModel,
    mut timer: Option<&mut LoginTimer>,
) -> std::result::Result<(TokenRes, TokenClaims), ApiError> {
    // 计时器是可选的：登录路径传入采集器，刷新路径不采集。
    let mut mark_stage = |label: &'static str| {
        if let Some(timer) = timer.as_deref_mut() {
            timer.mark(label);
        }
    };

    let codes = model::load_role_codes(&user.id)
        .await
        .map_err(|e| utils::db_error(e, "查询用户角色"))?;
    mark_stage("load_roles");
    let super_admin = model::is_super_admin(&codes);
    let permissions = model::load_permissions(&user.id, super_admin)
        .await
        .map_err(|e| utils::db_error(e, "查询用户权限"))?;
    mark_stage("load_permissions");

    let jwt = JWT::from_config();
    let user_id = user.id.to_string();
    // Access 与 Refresh 由**同一份** ClaimsBuilder 派生（框架推荐的签发入口）。
    // 这样后续追加自定义声明（如企业版的 `tenant_id`）时，刷新链路不会丢失载荷——
    // `create_refresh_token` 只带 `sub`，刷新后自定义声明会全部蒸发。
    let builder = || {
        ClaimsBuilder::new(user_id.clone())
            .username(&user.username)
            .roles(&codes)
            .permissions(&permissions)
    };
    let access_token = jwt.issue(builder()).map_err(|e| {
        tracing::error!(error = %e, "签发访问令牌失败");
        ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
    })?;
    let refresh_token = jwt.issue_refresh(builder()).map_err(|e| {
        tracing::error!(error = %e, "签发刷新令牌失败");
        ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
    })?;
    mark_stage("issue_jwt");

    // 签发接口只返回字符串，回读一次即可取得 jti / exp 供在线会话登记
    let claims = jwt.validate(&access_token).map_err(|e| {
        tracing::error!(error = %e, "回读访问令牌声明失败");
        ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
    })?;
    mark_stage("readback_claims");

    let token = TokenRes {
        access_token,
        refresh_token,
        token_type: TOKEN_TYPE.to_string(),
        expires_in: jwt.access_token_expire_secs(),
        user: Some(LoginUserRes {
            id: user_id,
            username: user.username.clone(),
            real_name: user.real_name.clone(),
        }),
    };
    Ok((token, claims))
}

/// 登记在线会话（登录 / 刷新成功后调用）。
///
/// # 参数
///
/// - `headers`：请求头（解析 User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：本次签发的访问令牌声明。
///
/// # 说明
///
/// 归属地仅读缓存（不外呼），保证请求路径不被网络 I/O 阻塞；登记失败只记告警，
/// **不影响登录结果**（在线列表属于旁路观测能力）。
async fn register_online_session(
    headers: &HeaderMap,
    connect_info: &SocketAddr,
    claims: &TokenClaims,
) {
    let Some(token_id) = claims.jti.as_deref() else {
        tracing::warn!(user_id = %claims.sub, "访问令牌缺少 jti，跳过在线会话登记");
        return;
    };
    let ip = utils::client_ip(headers, Some(connect_info));
    let (browser, os_type) = match utils::user_agent(headers) {
        Some(raw) => {
            let info = alun_utils::parse_user_agent(&raw);
            (Some(info.browser_type), Some(info.os_type))
        }
        None => (None, None),
    };
    let location = match ip {
        Some(addr) => crate::support::ipgeo::cached(addr).await,
        None => None,
    };

    let now = chrono::Utc::now();
    let expire_at = claims.exp as i64;
    let ttl = (expire_at - now.timestamp()).max(1) as u64;
    let session = OnlineSession {
        token_id: token_id.to_string(),
        user_id: claims.sub.clone(),
        username: claims.username.clone().unwrap_or_default(),
        ip: ip.map(|addr| addr.to_string()),
        location,
        browser,
        os_type,
        login_time: now,
        expire_at,
    };

    if let Err(e) = OnlineSessionModel::record_login(&session, ttl).await {
        tracing::warn!(error = %e, "登记在线会话失败（不影响登录结果）");
    }
}

/// 移除在线会话（登出时调用）。
///
/// # 参数
///
/// - `claims`：当前访问令牌声明。
async fn revoke_online_session(claims: &TokenClaims) {
    if let Some(token_id) = claims.jti.as_deref() {
        OnlineSessionModel::record_logout(token_id).await;
    }
}

/// 把扁平菜单列表组装为树。
///
/// # 参数
///
/// - `nodes`：扁平菜单投影列表（`MenuRes`）。
///
/// # 返回值
///
/// 根节点列表（已递归填充 `children`）。
fn build_menu_tree(nodes: Vec<MenuRes>) -> Vec<MenuRes> {
    utils::build_tree(
        nodes,
        |n| n.id.clone(),
        |n| n.parent_id.clone(),
        |n| n.sort_order,
        |n, children| n.children = children,
    )
}

/// 登录接口限流检查（按「用户名 + IP」维度）。
///
/// # 参数
///
/// - `username`：登录账号。
/// - `ip`：客户端 IP 文本（解析失败时为 `unknown`）。
///
/// # 返回值
///
/// 未超限返回 `Ok(())`。
///
/// # 异常情况
///
/// 超过窗口内最大尝试次数返回 429。缓存不可用时不限流（放行）。
async fn check_login_rate(username: &str, ip: &str) -> std::result::Result<(), ApiError> {
    let Some(store) = try_cache() else {
        return Ok(());
    };
    let key = format!("{}{}:{}", constants::CACHE_KEY_PREFIX_LOGIN, username, ip);
    let count = match Cache::incr(store, &key, 1).await {
        Ok(count) => count,
        Err(e) => {
            tracing::warn!(error = %e, "登录限流计数失败，本次放行");
            return Ok(());
        }
    };
    if count == 1
        && let Err(e) = Cache::set_ex(store, &key, &1i64, constants::LOGIN_RATE_WINDOW_SECS).await
    {
        tracing::warn!(error = %e, "设置登录限流窗口失败");
    }
    if count > constants::LOGIN_RATE_MAX_ATTEMPTS {
        return Err(ApiError::new(
            429,
            error_code::TOO_MANY_REQUESTS,
            "登录尝试过于频繁，请稍后再试",
        ));
    }
    Ok(())
}

/// 判断账号锁定是否已到期（到期视为可自动解锁）。
///
/// # 参数
///
/// - `user`：用户模型。
/// - `lock_minutes`：锁定后自动解锁的等待时长（分钟），取自 [`model::login_security_policy`]。
///
/// # 返回值
///
/// 处于锁定期内返回 `true`；未锁定或已过期返回 `false`。
fn is_locked_now(user: &SysUserModel, lock_minutes: i64) -> bool {
    if !user.is_locked {
        return false;
    }
    match user.lock_time {
        Some(lock_time) => chrono::Utc::now() < lock_time + chrono::Duration::minutes(lock_minutes),
        None => true,
    }
}

/// 记录一次登录失败：认证日志（`auth_logs` 失败行）+ 安全日志（`security_logs`）。
///
/// # 参数
///
/// - `headers`：请求头（派生 request_id / User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `username`：尝试登录的账号。
/// - `user_id`：命中的用户 ID（用户不存在时为 `None`）。
/// - `reason`：失败原因（落 `auth_logs.error_message` 与安全日志详情）。
/// - `timer`：耗时分段采集器（落库前输出分段耗时，失败路径同样可定位慢点）。
///
/// # 说明
///
/// 投递本身无 I/O，同步返回；落库由日志总线后台批量完成。
fn log_login_failure(
    headers: &HeaderMap,
    connect_info: &SocketAddr,
    username: &str,
    user_id: Option<uuid::Uuid>,
    reason: &str,
    timer: &LoginTimer,
) {
    log::record_auth(
        headers,
        connect_info,
        AuthLogInput {
            user_id,
            username: Some(username.to_string()),
            auth_type: AuthType::Login,
            auth_status: AuthStatus::Fail,
            error_message: Some(reason.to_string()),
            login_duration: Some(timer.total_ms() as i32),
        },
    );
    timer.report();
    log::record_security(
        headers,
        connect_info,
        SecurityLogInput {
            user_id,
            username: Some(username.to_string()),
            event_type: "LOGIN_FAIL".to_string(),
            detail: Some(serde_json::json!({ "reason": reason })),
        },
    );
}
