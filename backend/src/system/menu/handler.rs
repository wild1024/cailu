//! 菜单与权限点处理器：树查询、详情、增删改。
//!
//! 处理器承担**参数校验、业务编排、树形组装与响应组装**，并直接调用框架快捷方法
//! （`find_or_none` / `count_by` / `insert` / `update` / `delete` 等）；需要编写
//! SQL 的树查询集中在 [`crate::system::menu::model`]。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::net::SocketAddr;

use crate::shared::constants::{self, EnableStatus};
use crate::shared::error_code;
use crate::support::log;
use crate::system::menu::dto::{MenuCreateReq, MenuQuery, MenuRes, MenuUpdateReq};
use crate::system::menu::model::{self, MenuType, SysMenuModel};
use crate::utils;

// ==================== 接口 ====================

/// 菜单树查询（支持关键词 / 状态 / 类型 / 模块过滤）。
///
/// # 参数
///
/// - `query`：过滤条件。
///
/// # 返回值
///
/// 成功返回菜单树。
///
/// # 异常情况
///
/// 参数非法返回 400，数据库异常返回 500。
#[alun::permission(
    path = "/system/menu/tree",
    method = "GET",
    permission = "system:menu:list"
)]
#[alun::get("/system/menu/tree")]
pub async fn tree(Query(query): Query<MenuQuery>) -> ResResult<Vec<MenuRes>> {
    // 仅「无任何过滤条件」时走缓存：带过滤的结果集不确定，缓存价值低且易漏失效
    let no_filter = query.name.is_none()
        && query.status.is_none()
        && query.menu_type.is_none()
        && query.module_code.is_none();
    if no_filter && let Some(cached) = model::load_cached_tree().await {
        return Ok(Res::ok(cached));
    }

    let tree = query_menu_tree(&query).await?;
    if no_filter {
        model::cache_tree(&tree).await;
    }
    Ok(Res::ok(tree))
}

/// 菜单详情。
///
/// # 参数
///
/// - `id`：菜单 ID。
///
/// # 返回值
///
/// 成功返回菜单详情（`children` 为空）。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：菜单不存在。
#[alun::permission(
    path = "/system/menu/{id}",
    method = "GET",
    permission = "system:menu:list"
)]
#[alun::get("/system/menu/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<MenuRes> {
    let id = utils::require_uuid(&id, "id")?;
    let menu = SysMenuModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询菜单详情"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "菜单不存在"))?;
    Ok(Res::ok(MenuRes::from(&menu)))
}

/// 新建菜单。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（写入审计字段）。
/// - `req`：菜单请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`MENU_CREATE`，`after_data` 记录名称 / 上级 / 类型）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建菜单详情。
///
/// # 异常情况
///
/// 参数非法返回 400，父菜单不存在返回 404，数据库异常返回 500。
#[alun::permission(
    path = "/system/menu",
    method = "POST",
    permission = "system:menu:create"
)]
#[alun::post("/system/menu")]
pub async fn create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<MenuCreateReq>,
) -> ResResult<MenuRes> {
    let mut menu = build_model(&req, None, utils::current_user_uuid(&claims)).await?;
    menu.insert(db())
        .await
        .map_err(|e| utils::db_error(e, "新建菜单"))?;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "MENU_CREATE",
            "Menu",
            menu.id.to_string(),
            None,
            Some(serde_json::json!({
                "menu_name": menu.name,
                "parent_id": menu.parent_id.map(|v| v.to_string()),
                "menu_type": menu.menu_type.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(MenuRes::from(&menu)))
}

/// 更新菜单（全量）。
///
/// # 参数
///
/// - `id`：菜单 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `req`：菜单请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`MENU_UPDATE`，含名称 / 上级的变更前、后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的菜单详情。
///
/// # 异常情况
///
/// - 400：ID 或菜单类型非法；
/// - 404：菜单或父菜单不存在；
/// - 409：把菜单挂到自身之下；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/menu/{id}",
    method = "PUT",
    permission = "system:menu:update"
)]
#[alun::put("/system/menu/{id}")]
pub async fn update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<MenuUpdateReq>,
) -> ResResult<MenuRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = SysMenuModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待更新菜单"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "菜单不存在"))?;

    let menu = build_model(&req, Some(id), utils::current_user_uuid(&claims)).await?;
    if menu.parent_id == Some(id) {
        return Err(ApiError::new(
            409,
            error_code::RESOURCE_CONFLICT,
            "不能把菜单挂载到自身之下",
        ));
    }

    // 保留创建审计字段，仅更新业务字段
    let mut to_save = menu;
    to_save.created_at = existing.created_at;
    to_save.create_by = existing.create_by;
    to_save
        .update(db())
        .await
        .map_err(|e| utils::db_error(e, "更新菜单"))?;

    // 菜单的权限码 / 状态 / 标题等可能变更，令非超管在线用户令牌失效后重新登录，
    // 避免旧令牌内固化的权限快照与最新授权不一致
    crate::support::online::model::force_logout_all_except(&[constants::ADMIN_USERNAME]).await;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "MENU_UPDATE",
            "Menu",
            to_save.id.to_string(),
            Some(serde_json::json!({
                "menu_name": existing.name,
                "parent_id": existing.parent_id.map(|v| v.to_string()),
            })),
            Some(serde_json::json!({
                "menu_name": to_save.name,
                "parent_id": to_save.parent_id.map(|v| v.to_string()),
            })),
        ),
    );
    Ok(Res::ok(MenuRes::from(&to_save)))
}

/// 删除菜单（存在子节点时拒绝）。
///
/// # 参数
///
/// - `id`：菜单 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`MENU_DELETE`，`before_data` 记录被删菜单名称）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：菜单不存在；
/// - 409：存在子菜单；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/menu/{id}",
    method = "DELETE",
    permission = "system:menu:delete"
)]
#[alun::delete("/system/menu/{id}")]
pub async fn delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    // 删除前加载菜单，既做存在性校验，也为审计留存被删名称
    let menu = SysMenuModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待删除菜单"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "菜单不存在"))?;

    let children = SysMenuModel::count_by(db(), "parent_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "统计子菜单"))?;
    if children > 0 {
        return Err(ApiError::new(
            409,
            error_code::MENU_HAS_CHILDREN,
            "存在子菜单，无法删除",
        ));
    }

    // 事务：先清理角色-菜单授权绑定，再软删菜单，避免 sys_role_menu 残留孤儿绑定
    db().transaction(|tx| async move {
        tx.execute_with_params(
            "DELETE FROM sys_role_menu WHERE menu_id = ?",
            vec![Value::from(id)],
        )
        .await?;
        SysMenuModel::of(id).delete(&tx).await?;
        Ok(())
    })
    .await
    .map_err(|e| utils::db_error(e, "删除菜单"))?;

    // 菜单是权限来源：删除后令非超管在线用户的旧令牌失效，强制重新登录获取新权限，
    // 防止仍持有旧权限快照的令牌继续访问
    crate::support::online::model::force_logout_all_except(&[constants::ADMIN_USERNAME]).await;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "MENU_DELETE",
            "Menu",
            id.to_string(),
            Some(serde_json::json!({ "menu_name": menu.name })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

// ==================== 公共辅助 ====================

/// 按查询条件读取菜单，组装为树。
///
/// # 参数
///
/// - `query`：过滤条件（关键词 / 状态 / 类型 / 模块编码，均可选）。
///
/// # 返回值
///
/// 菜单树根节点列表。
///
/// # 异常情况
///
/// - 400：菜单类型取值非法；
/// - 500：数据库异常。
async fn query_menu_tree(query: &MenuQuery) -> std::result::Result<Vec<MenuRes>, ApiError> {
    // 提前校验菜单类型，避免非法取值静默返回空列表
    let filter = model::MenuTreeFilter {
        name: query.name.clone(),
        status: query.status,
        menu_type: query.menu_type_enum()?,
        module_code: query.module_code.clone(),
    };

    let menus = model::find_menu_list(&filter)
        .await
        .map_err(|e| utils::db_error(e, "查询菜单列表"))?;

    Ok(utils::build_tree(
        menus.iter().map(MenuRes::from).collect(),
        |n| n.id.clone(),
        |n| n.parent_id.clone(),
        |n| n.sort_order,
        |n, children| n.children = children,
    ))
}

/// 从请求构造菜单模型字段（新建 / 更新共用）。
///
/// # 参数
///
/// - `req`：菜单请求体。
/// - `id`：主键（更新时为既有 ID，新建时为 `None`）。
/// - `operator`：当前操作人 ID。
///
/// # 返回值
///
/// 可直接写入的菜单模型。
///
/// # 异常情况
///
/// - 400：菜单类型或父节点 ID 非法；
/// - 404：父节点不存在。
async fn build_model(
    req: &MenuCreateReq,
    id: Option<uuid::Uuid>,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<SysMenuModel, ApiError> {
    let menu_type = MenuType::from_i16(req.menu_type)?;
    let status = EnableStatus::from_optional_i16(req.status)?;
    let parent_id = utils::parse_optional_uuid(req.parent_id.as_deref(), "parent_id")?;

    if let Some(pid) = parent_id {
        let exists = SysMenuModel::find_or_none(db(), &pid)
            .await
            .map_err(|e| utils::db_error(e, "校验父菜单"))?
            .is_some();
        if !exists {
            return Err(ApiError::new(
                404,
                error_code::RESOURCE_NOT_FOUND,
                "父菜单不存在",
            ));
        }
    }

    let now = chrono::Utc::now();
    Ok(SysMenuModel {
        id: id.unwrap_or_else(uuid::Uuid::now_v7),
        name: req.name.trim().to_string(),
        menu_type,
        auth_code: req.auth_code.trim().to_string(),
        path: req.path.as_deref().map(str::trim).map(str::to_string),
        parent_id,
        component: req.component.as_deref().map(str::trim).map(str::to_string),
        sort_order: req.sort_order.unwrap_or(0),
        status,
        icon: req.icon.as_deref().map(str::trim).map(str::to_string),
        title: req.title.as_deref().map(str::trim).map(str::to_string),
        is_frame: req.is_frame.unwrap_or(false),
        is_cache: req.is_cache.unwrap_or(false),
        is_visible: req.is_visible.unwrap_or(true),
        hidden_expression: req
            .hidden_expression
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        module_code: req
            .module_code
            .as_deref()
            .map(str::trim)
            .map(str::to_string),
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    })
}
