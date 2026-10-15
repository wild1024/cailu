//! 角色管理处理器：分页、下拉、详情、增删改与菜单授权。
//!
//! 处理器承担**参数校验、业务编排与响应组装**，并直接调用框架快捷方法
//! （`find_or_none` / `find_by` / `count_by` / `delete_by` / `insert` / `update` /
//! `delete` 等）；需要编写 SQL 的查询（分页、下拉、编码唯一性校验）与领域行为
//! （菜单授权重建）集中在 [`crate::system::role::model`]。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::shared::constants::EnableStatus;
use crate::shared::{constants, error_code};
use crate::support::log;
use crate::system::role::dto::{
    RoleMenuAssignReq, RolePageQuery, RoleRes, RoleSaveReq, RoleUserPageQuery, RoleUserRes,
};
use crate::system::role::model::{self, SysRoleMenuModel, SysRoleModel};
use crate::system::user::model::SysUserRoleModel;
use crate::utils;

// ==================== 接口 ====================

/// 角色分页查询（支持角色名 / 编码模糊与状态过滤）。
///
/// # 参数
///
/// - `query`：分页与过滤条件。
///
/// # 返回值
///
/// 成功返回 `{list, total, page, page_size}`。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/system/role/page",
    method = "GET",
    permission = "system:role:list"
)]
#[alun::get("/system/role/page")]
pub async fn page(Query(query): Query<RolePageQuery>) -> ResResult<PageData<Vec<RoleRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);
    let filter = model::RolePageFilter {
        role_name: query.role_name,
        role_code: query.role_code,
        status: query.status,
    };

    let page_data = model::page_roles(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询角色"))?;

    let list: Vec<RoleRes> = page_data.list.iter().map(RoleRes::from).collect();
    Ok(Res::page(
        list,
        page_data.total,
        page_data.page,
        page_data.size,
    ))
}

/// 查询全部启用角色（按排序值升序，供下拉选择）。
///
/// # 返回值
///
/// 成功返回启用角色列表。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/system/role/list",
    method = "GET",
    permission = "system:role:list"
)]
#[alun::get("/system/role/list")]
pub async fn list() -> ResResult<Vec<RoleRes>> {
    // 启用角色下拉是高频热点查询，优先读缓存
    if let Some(cached) = model::load_cached_enabled_roles().await {
        return Ok(Res::ok(cached));
    }
    let roles = model::list_enabled_roles()
        .await
        .map_err(|e| utils::db_error(e, "查询启用角色"))?;
    let list: Vec<RoleRes> = roles.iter().map(RoleRes::from).collect();
    model::cache_enabled_roles(&list).await;
    Ok(Res::ok(list))
}

/// 角色详情。
///
/// # 参数
///
/// - `id`：角色 ID。
///
/// # 返回值
///
/// 成功返回角色详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：角色不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}",
    method = "GET",
    permission = "system:role:list"
)]
#[alun::get("/system/role/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<RoleRes> {
    let id = utils::require_uuid(&id, "id")?;
    let role = load_role(&id).await?;
    Ok(Res::ok(RoleRes::from(&role)))
}

/// 新建角色。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（写入审计字段）。
/// - `req`：角色请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`ROLE_CREATE`，`after_data` 记录角色编码 / 名称）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建角色详情。
///
/// # 异常情况
///
/// - 400：参数非法；
/// - 409：角色编码已存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role",
    method = "POST",
    permission = "system:role:create"
)]
#[alun::post("/system/role")]
pub async fn create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<RoleSaveReq>,
) -> ResResult<RoleRes> {
    let role_code = req.role_code.trim().to_string();
    ensure_role_code_unique(&role_code, None).await?;

    let now = chrono::Utc::now();
    let operator = utils::current_user_uuid(&claims);
    let mut role = SysRoleModel {
        id: uuid::Uuid::now_v7(),
        role_code,
        role_name: req.role_name.trim().to_string(),
        description: req.description.map(|v| v.trim().to_string()),
        status: EnableStatus::from_optional_i16(req.status)?,
        sort_order: req.sort_order.unwrap_or(0),
        remark: req.remark.map(|v| v.trim().to_string()),
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    };
    role.insert(db())
        .await
        .map_err(|e| utils::db_error(e, "新建角色"))?;
    model::invalidate_enabled_roles_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "ROLE_CREATE",
            "Role",
            role.id.to_string(),
            None,
            Some(serde_json::json!({
                "role_name": role.role_name,
                "role_code": role.role_code,
            })),
        ),
    );
    Ok(Res::ok(RoleRes::from(&role)))
}

/// 更新角色（全量业务字段；内置超级管理员角色不可禁用）。
///
/// # 参数
///
/// - `id`：角色 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（写入 `update_by`）。
/// - `req`：角色请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`ROLE_UPDATE`，含角色编码 / 名称的变更前、后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的角色详情。
///
/// # 异常情况
///
/// - 400：ID 或状态参数非法；
/// - 404：角色不存在；
/// - 409：角色编码已存在，或试图禁用内置超级管理员角色；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}",
    method = "PUT",
    permission = "system:role:update"
)]
#[alun::put("/system/role/{id}")]
pub async fn update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<RoleSaveReq>,
) -> ResResult<RoleRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = load_role(&id).await?;

    let role_code = req.role_code.trim().to_string();
    ensure_role_code_unique(&role_code, Some(id)).await?;

    let status = match req.status {
        Some(value) => EnableStatus::from_i16(value)?,
        None => existing.status,
    };
    // 内置角色（SUPER_ADMIN）短路全部权限校验，禁止禁用
    if existing.role_code == constants::SUPER_ADMIN_ROLE_CODE && status != EnableStatus::enabled {
        return Err(ApiError::new(
            409,
            error_code::ROLE_IN_USE,
            "内置超级管理员角色不可禁用",
        ));
    }

    let role = SysRoleModel {
        id,
        role_code,
        role_name: req.role_name.trim().to_string(),
        description: req.description.map(|v| v.trim().to_string()),
        status,
        sort_order: req.sort_order.unwrap_or(existing.sort_order),
        remark: req.remark.map(|v| v.trim().to_string()),
        // 保留创建审计字段，仅更新业务字段
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
        create_by: existing.create_by,
        update_by: utils::current_user_uuid(&claims),
        deleted_at: None,
    };
    role.update(db())
        .await
        .map_err(|e| utils::db_error(e, "更新角色"))?;

    model::invalidate_enabled_roles_cache().await;
    // 角色名称 / 状态变更会影响持有者权限快照，清理其主体上下文缓存
    crate::system::auth::model::invalidate_role_holders(&id).await;
    // 角色状态 / 名称等变更会影响持有该角色用户的权限快照，令非超管在线用户
    // 重新登录获取最新角色与权限
    crate::support::online::model::force_logout_all_except(&[constants::ADMIN_USERNAME]).await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "ROLE_UPDATE",
            "Role",
            role.id.to_string(),
            Some(serde_json::json!({
                "role_name": existing.role_name,
                "role_code": existing.role_code,
            })),
            Some(serde_json::json!({
                "role_name": role.role_name,
                "role_code": role.role_code,
            })),
        ),
    );
    Ok(Res::ok(RoleRes::from(&role)))
}

/// 删除角色（内置角色与已分配给用户的角色不可删除）。
///
/// # 参数
///
/// - `id`：角色 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`ROLE_DELETE`，`before_data` 记录被删角色编码 / 名称）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：角色不存在；
/// - 409：内置角色或角色已分配给用户；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}",
    method = "DELETE",
    permission = "system:role:delete"
)]
#[alun::delete("/system/role/{id}")]
pub async fn delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let role = load_role(&id).await?;
    if role.role_code == constants::SUPER_ADMIN_ROLE_CODE {
        return Err(ApiError::new(
            409,
            error_code::ROLE_IN_USE,
            "内置超级管理员角色不可删除",
        ));
    }

    let assigned = SysUserRoleModel::count_by(db(), "role_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "统计角色分配"))?;
    if assigned > 0 {
        return Err(ApiError::new(
            409,
            error_code::ROLE_IN_USE,
            "角色已分配给用户",
        ));
    }

    // 事务：软删角色 + 物理清理其菜单授权绑定，避免角色已删而授权残留
    db().transaction(|tx| async move {
        SysRoleModel::of(id).delete(&tx).await?;
        // 关联表为硬删表：物理删除该角色的全部菜单授权
        SysRoleMenuModel::delete_by(&tx, "role_id", (&id,)).await?;
        Ok(())
    })
    .await
    .map_err(|e| utils::db_error(e, "删除角色"))?;

    // 删除前已校验该角色未分配给任何用户，故无在线用户持有它，无需强制下线
    model::invalidate_enabled_roles_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "ROLE_DELETE",
            "Role",
            id.to_string(),
            Some(serde_json::json!({
                "role_name": role.role_name,
                "role_code": role.role_code,
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

/// 查询角色已授权的菜单 id 列表。
///
/// # 参数
///
/// - `id`：角色 ID。
///
/// # 返回值
///
/// 成功返回菜单 id 字符串列表。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：角色不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}/menus",
    method = "GET",
    permission = "system:role:list"
)]
#[alun::get("/system/role/{id}/menus")]
pub async fn menus(Path(id): Path<String>) -> ResResult<Vec<String>> {
    let id = utils::require_uuid(&id, "id")?;
    load_role(&id).await?;
    let bindings = SysRoleMenuModel::find_by(db(), "role_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "查询角色菜单授权"))?;
    Ok(Res::ok(
        bindings
            .into_iter()
            .map(|b| b.menu_id.to_string())
            .collect(),
    ))
}

/// 保存角色菜单授权（先清空全部授权，再逐条重建；空数组表示清空）。
///
/// # 参数
///
/// - `id`：角色 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
/// - `req`：菜单授权请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`ROLE_ASSIGN_MENU`，含授权前后的菜单 ID 列表）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 或菜单 ID 格式非法；
/// - 404：角色不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}/menus",
    method = "PUT",
    permission = "system:role:menu"
)]
#[alun::put("/system/role/{id}/menus")]
pub async fn assign_menus(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<RoleMenuAssignReq>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    load_role(&id).await?;

    let mut menu_ids = Vec::with_capacity(req.menu_ids.len());
    for raw in &req.menu_ids {
        menu_ids.push(utils::require_uuid(raw, "menu_ids")?);
    }

    // 记录授权变更前的菜单 id 列表，供审计 before 快照
    let before_menu_ids: Vec<String> = SysRoleMenuModel::find_by(db(), "role_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "查询角色菜单授权"))?
        .into_iter()
        .map(|b| b.menu_id.to_string())
        .collect();
    let after_menu_ids: Vec<String> = menu_ids.iter().map(|v| v.to_string()).collect();

    model::replace_role_menus(&id, &menu_ids)
        .await
        .map_err(|e| utils::db_error(e, "保存角色菜单授权"))?;

    // 角色菜单授权变更直接改变该角色用户的可访问权限，清理其主体上下文缓存
    crate::system::auth::model::invalidate_role_holders(&id).await;
    // 角色菜单授权变更直接改变该角色用户的可访问权限，令非超管在线用户令牌失效，
    // 强制重新登录以获取最新权限，避免旧令牌仍可访问已收回的功能
    crate::support::online::model::force_logout_all_except(&[constants::ADMIN_USERNAME]).await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "ROLE_ASSIGN_MENU",
            "Role",
            id.to_string(),
            Some(serde_json::json!({ "menu_ids": before_menu_ids })),
            Some(serde_json::json!({ "menu_ids": after_menu_ids })),
        ),
    );
    Ok(Res::ok_empty())
}

/// 查询持有该角色的用户（分页，供角色管理「角色用户列表」页签使用）。
///
/// # 参数
///
/// - `id`：角色 ID。
/// - `query`：分页条件。
///
/// # 返回值
///
/// 成功返回 `{list, total, page, page_size}`，列表项附带该用户的角色名称列表。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：角色不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/role/{id}/users",
    method = "GET",
    permission = "system:role:list"
)]
#[alun::get("/system/role/{id}/users")]
pub async fn users(
    Path(id): Path<String>,
    Query(query): Query<RoleUserPageQuery>,
) -> ResResult<PageData<Vec<RoleUserRes>>> {
    let id = utils::require_uuid(&id, "id")?;
    load_role(&id).await?;
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);
    let page_data = model::page_role_users(&id, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询角色用户"))?;

    // 单次 IN 查询批量取回本页用户的角色，按 user_id 聚合，避免 N+1
    let page_user_ids: Vec<uuid::Uuid> = page_data.list.iter().map(|u| u.id).collect();
    let role_rows = crate::system::user::model::find_user_roles_for_users(&page_user_ids)
        .await
        .map_err(|e| utils::db_error(e, "查询用户角色"))?;
    let mut role_names: HashMap<uuid::Uuid, Vec<String>> = HashMap::new();
    for row in role_rows {
        role_names
            .entry(row.user_id)
            .or_default()
            .push(row.role_name);
    }

    let list: Vec<RoleUserRes> = page_data
        .list
        .iter()
        .map(|u| RoleUserRes {
            id: u.id.to_string(),
            username: u.username.clone(),
            real_name: u.real_name.clone().unwrap_or_default(),
            mobile: u.mobile.clone().unwrap_or_default(),
            status: u.status.as_i16(),
            roles: role_names.get(&u.id).cloned().unwrap_or_default(),
        })
        .collect();
    Ok(Res::page(
        list,
        page_data.total,
        page_data.page,
        page_data.size,
    ))
}

// ==================== 公共辅助 ====================

/// 按主键加载角色，不存在时返回业务错误。
///
/// # 参数
///
/// - `id`：角色 ID。
///
/// # 返回值
///
/// 角色模型。
///
/// # 异常情况
///
/// - 404：角色不存在（`ROLE_NOT_FOUND`）；
/// - 500：数据库异常。
async fn load_role(id: &uuid::Uuid) -> std::result::Result<SysRoleModel, ApiError> {
    SysRoleModel::find_or_none(db(), id)
        .await
        .map_err(|e| utils::db_error(e, "查询角色"))?
        .ok_or_else(|| ApiError::new(404, error_code::ROLE_NOT_FOUND, "角色不存在"))
}

/// 校验角色编码唯一（软删表上仅比较未删除行）。
///
/// # 参数
///
/// - `role_code`：待校验的角色编码。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 编码未被占用时返回 `Ok(())`。
///
/// # 异常情况
///
/// - 409：角色编码已存在（`ROLE_CODE_EXISTS`）；
/// - 500：数据库异常。
async fn ensure_role_code_unique(
    role_code: &str,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<(), ApiError> {
    let count = model::count_roles_by_code(role_code, exclude)
        .await
        .map_err(|e| utils::db_error(e, "校验角色编码唯一性"))?;
    utils::ensure_unique(count, error_code::ROLE_CODE_EXISTS, "角色编码已存在")
}
