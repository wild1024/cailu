//! 认证模块的数据访问与领域逻辑。
//!
//! 按项目分层约定：**需要编写 SQL 的查询与领域行为集中在本文件**；仅调用框架
//! 快捷方法的简单读写（如按用户名 / 主键查用户、`of(...).set(...).update` 解锁）
//! 由 `handler.rs` 直接执行。认证过程读写的 `sys_user` / `sys_role` /
//! `sys_user_role` / `sys_menu` / `sys_role_menu` / `sys_config` 均复用各自模块的
//! 强类型模型，不重复定义表结构。

use alun::prelude::*;

use crate::shared::constants::{self, EnableStatus};
use crate::support::config::model::SysConfigModel;
use crate::system::menu::model::SysMenuModel;
use crate::system::role::model::SysRoleModel;
use crate::system::user::model::SysUserModel;

// ==================== 角色 / 权限 / 菜单 ====================

/// 用户角色列表缓存键。
///
/// # 参数
///
/// - `user_id`：用户 ID。
fn cache_key_user_roles(user_id: &uuid::Uuid) -> String {
    format!("{}{}", constants::CACHE_KEY_PREFIX_USER_ROLES, user_id)
}

/// 用户权限码列表缓存键。
fn cache_key_user_perms(user_id: &uuid::Uuid) -> String {
    format!("{}{}", constants::CACHE_KEY_PREFIX_USER_PERMS, user_id)
}

/// 用户可见菜单列表缓存键。
fn cache_key_user_menus(user_id: &uuid::Uuid) -> String {
    format!("{}{}", constants::CACHE_KEY_PREFIX_USER_MENUS, user_id)
}

/// 缓存里承载的「用户角色精简投影」。
///
/// `SysRoleModel` 未派生 serde（枚举字段仅实现 `TypeMapper`），不宜直接入缓存；
/// 而登录路径实际只用到 `role_code` 与 `status`（`role_codes` 就靠这两个字段），
/// 故用一个最小投影替代，既满足 serde 约束又避免把整个模型塞进缓存。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedRole {
    /// 角色编码。
    role_code: String,
    /// 是否启用（`role_codes` 仅保留启用角色）。
    enabled: bool,
}

impl CachedRole {
    /// 由角色模型投影。
    fn from_model(role: &SysRoleModel) -> Self {
        Self {
            role_code: role.role_code.clone(),
            enabled: role.status == EnableStatus::enabled,
        }
    }
}

/// 读取缓存值；缓存不可用或未命中返回 `None`（调用方回退查库）。
async fn cache_get<T>(key: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    let store = try_cache()?;
    Cache::get(store, key).await.ok().flatten()
}

/// 写入缓存；失败只告警，不影响主流程（缓存非主数据源）。
async fn cache_set<T>(key: &str, value: &T)
where
    T: serde::Serialize + Send + Sync + ?Sized,
{
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::set_ex(store, key, value, constants::CACHE_PRINCIPAL_TTL_SECS).await {
        tracing::warn!(key, error = %e, "写入主体上下文缓存失败");
    }
}

/// 失效指定用户的角色 / 权限 / 菜单三项缓存。
///
/// 授权类变更（用户角色分配、角色菜单授权、角色启停、菜单增删改）后调用。
/// 缓存未初始化时直接跳过（无缓存即无脏读）。
///
/// # 参数
///
/// - `user_id`：用户 ID。
pub(crate) async fn invalidate_user_principal(user_id: &uuid::Uuid) {
    let Some(store) = try_cache() else {
        return;
    };
    for key in [
        cache_key_user_roles(user_id),
        cache_key_user_perms(user_id),
        cache_key_user_menus(user_id),
    ] {
        if let Err(e) = Cache::del(store, &key).await {
            tracing::warn!(key, error = %e, "失效主体上下文缓存失败");
        }
    }
}

/// 失效多个用户的主体上下文缓存（角色被批量改授权时使用）。
///
/// # 参数
///
/// - `user_ids`：用户 ID 列表。
pub(crate) async fn invalidate_users_principal(user_ids: &[uuid::Uuid]) {
    for user_id in user_ids {
        invalidate_user_principal(user_id).await;
    }
}

/// 按角色 ID 失效持有该角色的**全部**用户的主体上下文缓存。
///
/// 角色菜单授权变化后，受影响的不是单个用户，而是所有持有该角色的用户。
/// 为避免漏失效导致越权，这里按 `sys_user_role` 反查全部用户逐个清理
/// （授权变更是低频操作，反查成本可接受）。
///
/// # 参数
///
/// - `role_id`：角色 ID。
pub(crate) async fn invalidate_role_holders(role_id: &uuid::Uuid) {
    let rows = match db()
        .find_with_params(
            "SELECT DISTINCT user_id FROM sys_user_role WHERE role_id = ?",
            vec![Value::from(role_id)],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(?role_id, error = %e, "反查角色持有者以失效缓存失败");
            return;
        }
    };

    let mut user_ids: Vec<uuid::Uuid> = Vec::with_capacity(rows.len());
    for row in rows {
        match row.get_typed::<uuid::Uuid>("user_id") {
            Ok(id) => user_ids.push(id),
            Err(e) => tracing::warn!(?role_id, error = %e, "解析角色持有者 ID 失败，跳过"),
        }
    }
    if !user_ids.is_empty() {
        tracing::debug!(?role_id, users = user_ids.len(), "失效角色持有者的主体上下文缓存");
    }
    invalidate_users_principal(&user_ids).await;
}

/// 查询用户当前生效的角色编码（仅启用角色），带缓存。
///
/// 登录 / 刷新 / `me` / `menus` 都要用角色编码判定超级管理员，而原查询是
/// `sys_role` 的 `IN (SELECT ... sys_user_role)` 子查询 —— 属最高频读路径，故缓存。
/// 缓存内只保留 [`CachedRole`]（编码 + 启用位），登录路径不需要其余字段。
///
/// # 参数
///
/// - `user_id`：用户 ID。
///
/// # 返回值
///
/// 启用角色的 `role_code` 列表（顺序与数据库返回一致）。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]；缓存异常不影响结果（回退查库）。
pub(crate) async fn load_role_codes(
    user_id: &uuid::Uuid,
) -> std::result::Result<Vec<String>, DbError> {
    let key = cache_key_user_roles(user_id);
    if let Some(cached) = cache_get::<Vec<CachedRole>>(&key).await {
        return Ok(cached.into_iter().filter(|r| r.enabled).map(|r| r.role_code).collect());
    }
    let roles = load_roles_uncached(user_id).await?;
    let projection = roles.iter().map(CachedRole::from_model).collect::<Vec<_>>();
    cache_set(&key, &projection).await;
    Ok(projection
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| r.role_code)
        .collect())
}

/// 查询用户当前生效的角色（不走缓存，供内部装载 / 回填复用）。
async fn load_roles_uncached(
    user_id: &uuid::Uuid,
) -> std::result::Result<Vec<SysRoleModel>, DbError> {
    SysRoleModel::find_by(
        db(),
        "id IN (SELECT role_id FROM sys_user_role \
                WHERE user_id = ? AND (expire_time IS NULL OR expire_time > now()))",
        (user_id,),
    )
    .await
}

/// 判断角色编码中是否包含超级管理员。
///
/// # 参数
///
/// - `codes`：角色编码列表。
///
/// # 返回值
///
/// 含 [`constants::SUPER_ADMIN_ROLE_CODE`] 时返回 `true`。
pub(crate) fn is_super_admin(codes: &[String]) -> bool {
    codes.iter().any(|c| c == constants::SUPER_ADMIN_ROLE_CODE)
}

/// 查询用户权限码列表。
///
/// # 参数
///
/// - `user_id`：用户 ID。
/// - `super_admin`：是否超级管理员（是则短路为 `["*:*:*"]`，不查授权表）。
///
/// # 返回值
///
/// 去重后的权限码列表。
///
/// # 异常情况
///
/// 数据库异常或行缺少 `auth_code` 列时返回 [`DbError`]。
pub(crate) async fn load_permissions(
    user_id: &uuid::Uuid,
    super_admin: bool,
) -> std::result::Result<Vec<String>, DbError> {
    if super_admin {
        // 通配码供**框架的** `#[alun::permission]` 校验层短路使用；业务侧的超管判定
        // （数据权限 / 脱敏 / 菜单）一律以角色编码为准（见 shared/principal.rs），
        // 不读取本通配码，故二者不构成双真值源。
        return Ok(vec!["*:*:*".to_string()]);
    }

    // 权限码由「用户 → 角色 → 菜单」三表 JOIN 得出，是登录后最高频的读路径，故缓存。
    let key = cache_key_user_perms(user_id);
    if let Some(cached) = cache_get(&key).await {
        return Ok(cached);
    }
    let permissions = load_permissions_uncached(user_id).await?;
    cache_set(&key, &permissions).await;
    Ok(permissions)
}

/// 查询用户权限码列表（不走缓存，供内部装载 / 回填复用）。
async fn load_permissions_uncached(
    user_id: &uuid::Uuid,
) -> std::result::Result<Vec<String>, DbError> {
    let rows = db()
        .find_with_params(
            "SELECT DISTINCT m.auth_code AS auth_code FROM sys_menu m \
             INNER JOIN sys_role_menu rm ON rm.menu_id = m.id \
             INNER JOIN sys_user_role ur ON ur.role_id = rm.role_id \
             WHERE ur.user_id = ? AND m.deleted_at IS NULL AND m.status = 1 \
               AND m.auth_code <> '' \
               AND (ur.expire_time IS NULL OR ur.expire_time > now())",
            vec![Value::from(user_id)],
        )
        .await?;

    let mut permissions: Vec<String> = Vec::with_capacity(rows.len());
    for row in rows {
        let code: String = row.get_typed("auth_code")?;
        if !code.is_empty() && !permissions.contains(&code) {
            permissions.push(code);
        }
    }
    Ok(permissions)
}

/// 查询用户可见菜单（仅目录与菜单，不含按钮），超级管理员为全部启用菜单，带缓存。
///
/// 返回 `MenuRes` 而非 `SysMenuModel`：后者未派生 serde，而调用方本来就要立即
/// 转成 `MenuRes` 再建树，直接缓存投影可省去一次转换，也满足缓存的 serde 约束。
///
/// # 参数
///
/// - `user_id`：用户 ID。
/// - `super_admin`：是否超级管理员。
///
/// # 返回值
///
/// 已按 `sort_order` 升序的扁平菜单列表。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]；缓存异常不影响结果（回退查库）。
pub(crate) async fn load_user_menus(
    user_id: &uuid::Uuid,
    super_admin: bool,
) -> std::result::Result<Vec<crate::system::menu::dto::MenuRes>, DbError> {
    // 超管的可见菜单是「全部启用目录与菜单」，与用户无关；但仍按用户键缓存，
    // 避免超管与非超管共用一键时读到彼此的结果。
    let key = cache_key_user_menus(user_id);
    if let Some(cached) = cache_get(&key).await {
        return Ok(cached);
    }
    let menus = load_user_menus_uncached(user_id, super_admin).await?;
    let nodes: Vec<crate::system::menu::dto::MenuRes> = menus
        .iter()
        .map(crate::system::menu::dto::MenuRes::from)
        .collect();
    cache_set(&key, &nodes).await;
    Ok(nodes)
}

/// 查询用户可见菜单（不走缓存，供内部装载 / 回填复用）。
async fn load_user_menus_uncached(
    user_id: &uuid::Uuid,
    super_admin: bool,
) -> std::result::Result<Vec<SysMenuModel>, DbError> {
    if super_admin {
        SysMenuModel::find_by(
            db(),
            "menu_type IN (1, 2) AND status = 1 ORDER BY sort_order ASC",
            (),
        )
        .await
    } else {
        SysMenuModel::find_by(
            db(),
            "menu_type IN (1, 2) AND status = 1 \
             AND id IN (SELECT rm.menu_id FROM sys_role_menu rm \
                        INNER JOIN sys_user_role ur ON ur.role_id = rm.role_id \
                        WHERE ur.user_id = ? \
                          AND (ur.expire_time IS NULL OR ur.expire_time > now())) \
             ORDER BY sort_order ASC",
            (user_id,),
        )
        .await
    }
}

// ==================== 登录状态维护 ====================

/// 记录一次登录失败：累加失败计数，达到阈值时锁定账号。
///
/// # 参数
///
/// - `user_id`：用户 ID。
/// - `fail_count`：累加后的失败次数。
/// - `is_lock`：是否达到锁定阈值。
/// - `now`：失败发生时间（锁定时写入 `lock_time`）。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn record_login_failure(
    user_id: &uuid::Uuid,
    fail_count: i16,
    is_lock: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> std::result::Result<(), DbError> {
    let mut changeset = SysUserModel::of(user_id).set(SysUserModel::login_fail_count(), fail_count);
    if is_lock {
        changeset = changeset
            .set(SysUserModel::is_locked(), true)
            .set(SysUserModel::lock_time(), Some(now));
    }
    changeset.update(db()).await.map(|_| ())
}

// ==================== 登录安全策略 ====================

/// 读取登录安全参数：连续失败锁定阈值与自动解锁时长。
///
/// 取值优先来自 `sys_config`（[`constants::CONFIG_KEY_LOGIN_MAX_FAIL`] /
/// [`constants::CONFIG_KEY_LOGIN_LOCK_MINUTES`]，由种子迁移 `S__config_data` 写入）；
/// 参数缺失、被停用或解析失败时回退 [`constants`] 中的默认常量，因此参数不全
/// 不会导致登录不可用。
///
/// # 返回值
///
/// `(连续失败锁定阈值, 自动解锁时长分钟)`。
pub(crate) async fn login_security_policy() -> (i16, i64) {
    let max_fail = SysConfigModel::read_i64(constants::CONFIG_KEY_LOGIN_MAX_FAIL)
        .await
        .and_then(|value| i16::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(constants::MAX_LOGIN_FAIL_COUNT);
    let lock_minutes = SysConfigModel::read_i64(constants::CONFIG_KEY_LOGIN_LOCK_MINUTES)
        .await
        .filter(|value| *value > 0)
        .unwrap_or(constants::LOCK_MINUTES);
    (max_fail, lock_minutes)
}
