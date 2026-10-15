//! 部门机构处理器：树查询、详情、增删改。
//!
//! 处理器承担**参数校验、业务编排、树形组装与响应组装**，并直接调用框架快捷方法
//! （`find_or_none` / `count_by` / `insert` / `update` / `delete` 等）；需要编写
//! SQL 的查询（树查询、编码唯一性校验）集中在 [`crate::system::dept::model`]。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::net::SocketAddr;

use crate::shared::constants::EnableStatus;
use crate::shared::error_code;
use crate::support::log;
use crate::system::dept::dto::{
    DeptDetailRes, DeptMigrateReq, DeptNode, DeptSaveReq, DeptTreeQuery,
};
use crate::system::dept::model::{self, SysDeptModel};
use crate::system::user::model::SysUserModel;
use crate::utils;

// ==================== 接口 ====================

/// 部门树查询（支持部门名模糊与状态过滤）。
///
/// # 参数
///
/// - `query`：过滤条件。
///
/// # 返回值
///
/// 成功返回按 `sort_order` 升序组装的部门树根节点列表。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/system/dept/tree",
    method = "GET",
    permission = "system:dept:list"
)]
#[alun::get("/system/dept/tree")]
pub async fn tree(Query(query): Query<DeptTreeQuery>) -> ResResult<Vec<DeptNode>> {
    // 仅「无任何过滤条件」的全量树走缓存；带名称 / 状态过滤的结果不缓存
    let no_filter = query.dept_name.is_none() && query.status.is_none();
    if no_filter && let Some(cached) = model::load_cached_tree().await {
        return Ok(Res::ok(cached));
    }

    let filter = model::DeptTreeFilter {
        dept_name: query.dept_name,
        status: query.status,
    };
    let depts = model::find_dept_list(&filter)
        .await
        .map_err(|e| utils::db_error(e, "查询部门列表"))?;

    // 批量解析负责人显示名（real_name 优先，回退 username），避免逐部门 N+1 查询
    let leader_ids: Vec<uuid::Uuid> = depts.iter().filter_map(|d| d.leader_id).collect();
    let leader_names = model::find_user_display_names(&leader_ids)
        .await
        .map_err(|e| utils::db_error(e, "查询部门负责人"))?;

    let mut nodes: Vec<DeptNode> = depts.iter().map(DeptNode::from).collect();
    for node in &mut nodes {
        if let Some(id_str) = node.leader_id.as_deref()
            && let Ok(id) = uuid::Uuid::parse_str(id_str)
            && let Some(name) = leader_names.get(&id)
        {
            node.leader_name = Some(name.clone());
        }
    }

    let nodes = utils::build_tree(
        nodes,
        |n| n.id.clone(),
        |n| n.parent_id.clone(),
        |n| n.sort_order,
        |n, children| n.children = children,
    );

    if no_filter {
        model::cache_tree(&nodes).await;
    }
    Ok(Res::ok(nodes))
}

/// 部门详情（不含 `children`）。
///
/// # 参数
///
/// - `id`：部门 ID。
///
/// # 返回值
///
/// 成功返回部门详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：部门不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/dept/{id}",
    method = "GET",
    permission = "system:dept:list"
)]
#[alun::get("/system/dept/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<DeptDetailRes> {
    let id = utils::require_uuid(&id, "id")?;
    let dept = load_dept(&id).await?;
    Ok(Res::ok(DeptDetailRes::from(&dept)))
}

/// 新建部门（自动维护 `dept_path`）。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（写入审计字段）。
/// - `req`：部门请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DEPT_CREATE`，`after_data` 记录编码 / 名称 / 上级）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建部门详情。
///
/// # 异常情况
///
/// - 400：参数非法；
/// - 404：父部门不存在；
/// - 409：部门编码已存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/dept",
    method = "POST",
    permission = "system:dept:create"
)]
#[alun::post("/system/dept")]
pub async fn create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DeptSaveReq>,
) -> ResResult<DeptDetailRes> {
    let dept_code = req.dept_code.trim().to_string();
    ensure_dept_code_unique(&dept_code, None).await?;

    let id = uuid::Uuid::now_v7();
    let parent_id = utils::parse_optional_uuid(req.parent_id.as_deref(), "parent_id")?;
    let dept_name = req.dept_name.trim().to_string();
    ensure_dept_name_unique(&dept_name, parent_id, None).await?;
    let dept_path = match parent_id {
        Some(pid) => {
            let parent = load_parent(&pid).await?;
            Some(join_child_path(parent.dept_path.as_deref(), &id))
        }
        None => Some(format!("/{id}/")),
    };

    let now = chrono::Utc::now();
    let operator = utils::current_user_uuid(&claims);
    let mut dept = SysDeptModel {
        id,
        dept_code,
        dept_name,
        parent_id,
        dept_path,
        leader_id: utils::parse_optional_uuid(req.leader_id.as_deref(), "leader_id")?,
        phone: req.phone.map(|v| v.trim().to_string()),
        email: req.email.map(|v| v.trim().to_string()),
        status: EnableStatus::from_optional_i16(req.status)?,
        sort_order: req.sort_order.unwrap_or(0),
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    };
    dept.insert(db())
        .await
        .map_err(|e| utils::db_error(e, "新建部门"))?;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DEPT_CREATE",
            "Dept",
            dept.id.to_string(),
            None,
            Some(serde_json::json!({
                "dept_code": dept.dept_code,
                "dept_name": dept.dept_name,
                "parent_id": dept.parent_id.map(|v| v.to_string()),
            })),
        ),
    );
    Ok(Res::ok(DeptDetailRes::from(&dept)))
}

/// 更新部门（重算自身 `dept_path`；禁止挂到自身或其子部门下）。
///
/// # 参数
///
/// - `id`：部门 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（写入 `update_by`）。
/// - `req`：部门请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DEPT_UPDATE`，含变更前 / 后快照）；日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的部门详情。
///
/// # 异常情况
///
/// - 400：ID 或状态参数非法；
/// - 404：部门或父部门不存在；
/// - 409：部门编码已存在，或试图把部门挂到自身 / 其子部门之下；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/dept/{id}",
    method = "PUT",
    permission = "system:dept:update"
)]
#[alun::put("/system/dept/{id}")]
pub async fn update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DeptSaveReq>,
) -> ResResult<DeptDetailRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = load_dept(&id).await?;

    let dept_code = req.dept_code.trim().to_string();
    ensure_dept_code_unique(&dept_code, Some(id)).await?;

    let parent_id = utils::parse_optional_uuid(req.parent_id.as_deref(), "parent_id")?;
    if parent_id == Some(id) {
        return Err(ApiError::new(
            409,
            error_code::RESOURCE_CONFLICT,
            "不能把部门挂到自身或其子部门下",
        ));
    }
    let dept_name = req.dept_name.trim().to_string();
    ensure_dept_name_unique(&dept_name, parent_id, Some(id)).await?;
    let dept_path = match parent_id {
        Some(pid) => {
            let parent = load_parent(&pid).await?;
            let parent_path = parent
                .dept_path
                .clone()
                .unwrap_or_else(|| format!("/{pid}/"));
            // 父部门路径中含有「自身 id」说明自身是父部门的祖先，即父部门位于自身子树内
            if parent_path.contains(&format!("/{id}/")) {
                return Err(ApiError::new(
                    409,
                    error_code::RESOURCE_CONFLICT,
                    "不能把部门挂到自身或其子部门下",
                ));
            }
            Some(join_child_path(Some(&parent_path), &id))
        }
        None => Some(format!("/{id}/")),
    };

    // 仅重算自身 dept_path，不改写已存在子部门的历史路径（已知限制：
    // 搬迁部门后其后代路径不再自动同步，范围查询可能遗漏，后续版本再补齐）。
    let dept = SysDeptModel {
        id,
        dept_code,
        dept_name,
        parent_id,
        dept_path,
        leader_id: utils::parse_optional_uuid(req.leader_id.as_deref(), "leader_id")?,
        phone: req.phone.map(|v| v.trim().to_string()),
        email: req.email.map(|v| v.trim().to_string()),
        status: match req.status {
            Some(value) => EnableStatus::from_i16(value)?,
            None => existing.status,
        },
        sort_order: req.sort_order.unwrap_or(existing.sort_order),
        // 保留创建审计字段，仅更新业务字段
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
        create_by: existing.create_by,
        update_by: utils::current_user_uuid(&claims),
        deleted_at: None,
    };
    dept.update(db())
        .await
        .map_err(|e| utils::db_error(e, "更新部门"))?;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DEPT_UPDATE",
            "Dept",
            dept.id.to_string(),
            Some(serde_json::json!({
                "dept_code": existing.dept_code,
                "dept_name": existing.dept_name,
                "parent_id": existing.parent_id.map(|v| v.to_string()),
            })),
            Some(serde_json::json!({
                "dept_code": dept.dept_code,
                "dept_name": dept.dept_name,
                "parent_id": dept.parent_id.map(|v| v.to_string()),
            })),
        ),
    );
    Ok(Res::ok(DeptDetailRes::from(&dept)))
}

/// 删除部门（存在子部门或归属用户时拒绝）。
///
/// # 参数
///
/// - `id`：部门 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DEPT_DELETE`，`before_data` 记录被删部门快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：部门不存在；
/// - 409：存在子部门（`DEPT_HAS_CHILDREN`）或部门下存在用户；
/// - 500：数据库异常。
#[alun::permission(
    path = "/system/dept/{id}",
    method = "DELETE",
    permission = "system:dept:delete"
)]
#[alun::delete("/system/dept/{id}")]
pub async fn delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let dept = load_dept(&id).await?;

    let children = SysDeptModel::count_by(db(), "parent_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "统计子部门"))?;
    if children > 0 {
        return Err(ApiError::new(
            409,
            error_code::DEPT_HAS_CHILDREN,
            "存在子部门，无法删除",
        ));
    }

    let users = SysUserModel::count_by(db(), "dept_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "统计部门用户"))?;
    if users > 0 {
        return Err(ApiError::new(
            409,
            error_code::RESOURCE_CONFLICT,
            "部门下存在用户，无法删除",
        ));
    }

    SysDeptModel::of(id)
        .delete(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "删除部门"))?;
    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DEPT_DELETE",
            "Dept",
            id.to_string(),
            Some(serde_json::json!({
                "dept_code": dept.dept_code,
                "dept_name": dept.dept_name,
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

/// 迁移部门用户后删除部门（事务保证原子性）。
///
/// 与 [`delete`] 不同，本接口允许部门下仍有用户：先在同一事务内把这些用户的
/// `dept_id` 统一改挂到目标部门（`migrate_to_dept_id` 为空则置为「无部门」），
/// 再软删本部门。存在子部门时仍拒绝删除（需先处理下级机构）。
///
/// # 参数
///
/// - `id`：待删除部门 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
/// - `req`：迁移目标部门请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DEPT_MIGRATE_DELETE`，`before_data` 记录被删部门与迁移目标）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：部门或目标部门不存在；
/// - 409：存在子部门，或目标部门就是被删除部门自身；
/// - 500：数据库异常（事务整体回滚，不会出现用户已迁移而部门未删的中间态）。
#[alun::permission(
    path = "/system/dept/{id}/migrate-delete",
    method = "POST",
    permission = "system:dept:delete"
)]
#[alun::post("/system/dept/{id}/migrate-delete")]
pub async fn migrate_and_delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DeptMigrateReq>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let dept = load_dept(&id).await?;

    // 存在子部门时不允许删除（迁移仅处理本部门直属用户，不连带处理下级机构）
    let children = SysDeptModel::count_by(db(), "parent_id", (&id,))
        .await
        .map_err(|e| utils::db_error(e, "统计子部门"))?;
    if children > 0 {
        return Err(ApiError::new(
            409,
            error_code::DEPT_HAS_CHILDREN,
            "存在子部门，请先迁移或删除子部门",
        ));
    }

    // 校验目标部门：可空（迁出至无部门）；非空时必须存在且不能是自身
    let target_id =
        utils::parse_optional_uuid(req.migrate_to_dept_id.as_deref(), "migrate_to_dept_id")?;
    if let Some(target) = target_id {
        if target == id {
            return Err(ApiError::new(
                409,
                error_code::RESOURCE_CONFLICT,
                "目标部门不能是被删除部门自身",
            ));
        }
        if SysDeptModel::find_or_none(db(), &target)
            .await
            .map_err(|e| utils::db_error(e, "查询目标部门"))?
            .is_none()
        {
            return Err(ApiError::new(
                404,
                error_code::DEPT_NOT_FOUND,
                "目标部门不存在",
            ));
        }
    }

    // 事务：先迁移用户，再软删部门；任一步失败整体回滚
    db().transaction(|tx| async move {
        match target_id {
            Some(target) => {
                tx.execute_with_params(
                    "UPDATE sys_user SET dept_id = ?, updated_at = now() \
                         WHERE dept_id = ? AND deleted_at IS NULL",
                    vec![Value::from(target), Value::from(id)],
                )
                .await?;
            }
            None => {
                tx.execute_with_params(
                    "UPDATE sys_user SET dept_id = NULL, updated_at = now() \
                         WHERE dept_id = ? AND deleted_at IS NULL",
                    vec![Value::from(id)],
                )
                .await?;
            }
        }
        SysDeptModel::of(id).delete(&tx).await?;
        Ok(())
    })
    .await
    .map_err(|e| utils::db_error(e, "迁移用户并删除部门"))?;

    model::invalidate_tree_cache().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DEPT_MIGRATE_DELETE",
            "Dept",
            id.to_string(),
            Some(serde_json::json!({
                "dept_code": dept.dept_code,
                "dept_name": dept.dept_name,
                "migrate_to_dept_id": target_id.map(|v| v.to_string()),
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

// ==================== 公共辅助 ====================

/// 拼接子节点层级路径（父路径 + 自身 id + `/`）。
///
/// # 参数
///
/// - `parent_path`：父部门 `dept_path`；为 `None` 时按根路径处理。
/// - `id`：子部门（自身）id。
///
/// # 返回值
///
/// 形如 `/{父路径}/{自身id}/` 的层级路径。
fn join_child_path(parent_path: Option<&str>, id: &uuid::Uuid) -> String {
    match parent_path {
        Some(p) if p.ends_with('/') => format!("{p}{id}/"),
        Some(p) => format!("{p}/{id}/"),
        None => format!("/{id}/"),
    }
}

/// 按主键加载部门，不存在时返回业务错误。
///
/// # 参数
///
/// - `id`：部门 ID。
///
/// # 返回值
///
/// 部门模型。
///
/// # 异常情况
///
/// - 404：部门不存在（`RESOURCE_NOT_FOUND`）；
/// - 500：数据库异常。
async fn load_dept(id: &uuid::Uuid) -> std::result::Result<SysDeptModel, ApiError> {
    SysDeptModel::find_or_none(db(), id)
        .await
        .map_err(|e| utils::db_error(e, "查询部门"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "部门不存在"))
}

/// 加载父部门，不存在时返回业务错误。
///
/// # 参数
///
/// - `id`：父部门 ID。
///
/// # 返回值
///
/// 父部门模型。
///
/// # 异常情况
///
/// - 404：父部门不存在；
/// - 500：数据库异常。
async fn load_parent(id: &uuid::Uuid) -> std::result::Result<SysDeptModel, ApiError> {
    SysDeptModel::find_or_none(db(), id)
        .await
        .map_err(|e| utils::db_error(e, "查询父部门"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "父部门不存在"))
}

/// 校验部门编码唯一（软删表上仅比较未删除行）。
///
/// # 参数
///
/// - `dept_code`：待校验的部门编码。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 编码未被占用时返回 `Ok(())`。
///
/// # 异常情况
///
/// - 409：部门编码已存在（`DEPT_CODE_EXISTS`）；
/// - 500：数据库异常。
async fn ensure_dept_code_unique(
    dept_code: &str,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<(), ApiError> {
    let count = model::count_depts_by_code(dept_code, exclude)
        .await
        .map_err(|e| utils::db_error(e, "校验部门编码唯一性"))?;
    utils::ensure_unique(count, error_code::DEPT_CODE_EXISTS, "部门编码已存在")
}

/// 校验同一上级部门下名称唯一（不同层级允许同名）。
///
/// # 参数
///
/// - `dept_name`：待校验的部门名称。
/// - `parent_id`：上级部门 ID；`None` 表示根部门。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 同级无同名部门时返回 `Ok(())`。
///
/// # 异常情况
///
/// - 409：同级部门下名称已存在（`DEPT_NAME_EXISTS`）；
/// - 500：数据库异常。
async fn ensure_dept_name_unique(
    dept_name: &str,
    parent_id: Option<uuid::Uuid>,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<(), ApiError> {
    let count = model::count_depts_by_name(dept_name, parent_id, exclude)
        .await
        .map_err(|e| utils::db_error(e, "校验部门同级名称唯一性"))?;
    utils::ensure_unique(count, error_code::DEPT_NAME_EXISTS, "同级部门下名称已存在")
}
