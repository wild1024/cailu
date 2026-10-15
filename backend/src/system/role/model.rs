//! 角色与「角色-菜单」授权的强类型模型。
//!
//! 对应迁移 `V0002__create_sys_role` 与 `V0005__create_sys_role_menu`。
//!
//! - [`SysRoleModel`]：软删表，含 `created_at` / `updated_at`，框架自动挂载
//!   `TimestampHook`；
//! - [`SysRoleMenuModel`]：关联表，仅含 `created_at`，硬删 +
//!   `#[model(no_timestamps, hard_delete)]`。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（分页、下拉、唯一性校验）与领域行为（菜单授权重建）集中在
//! 本文件；仅调用框架快捷方法（`find_or_none` / `find_by` / `count_by` /
//! `insert` / `update` / `delete` / `delete_by` 等）的简单读写由 `handler.rs`
//! 直接执行。

use alun::prelude::*;

use crate::shared::constants;
use crate::shared::constants::EnableStatus;
use crate::system::user::model::SysUserModel;
use crate::utils;

/// `sys_role` 表对应的强类型模型（角色）。
///
/// # 唯一性与软删
///
/// `role_code` 的部分唯一索引带 `WHERE deleted_at IS NULL`，保证删除后可重建
/// 同名编码；内置角色（如 `SUPER_ADMIN`）由业务层禁止删除。
#[derive(Debug, Clone, Model)]
#[table("sys_role")]
pub struct SysRoleModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 角色编码，软删表上的部分唯一列（如 `SUPER_ADMIN`）。
    pub role_code: String,
    /// 角色名称。
    pub role_name: String,
    /// 角色描述。
    pub description: Option<String>,
    /// 角色状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 排序号，升序展示。
    pub sort_order: i32,
    /// 备注。
    pub remark: Option<String>,
    /// 创建时间，由 `TimestampHook` 维护。
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间，由 `TimestampHook` 维护。
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 创建人 ID。
    pub create_by: Option<uuid::Uuid>,
    /// 更新人 ID。
    pub update_by: Option<uuid::Uuid>,
    /// 软删标记（`NULL` = 未删除）。
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// `sys_role_menu` 表对应的强类型模型（角色-菜单授权）。
///
/// # 硬删关联表
///
/// 表上仅有 `created_at`，唯一约束 `(role_id, menu_id)`；派生标注
/// `#[model(no_timestamps, hard_delete)]`：不自动挂载时间戳 Hook，删除为物理删除
/// （重新授权即整批重建记录）。
#[derive(Debug, Clone, Model)]
#[table("sys_role_menu")]
#[model(no_timestamps, hard_delete)]
pub struct SysRoleMenuModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 角色 ID，引用 `sys_role(id)`。
    pub role_id: uuid::Uuid,
    /// 菜单 ID，引用 `sys_menu(id)`。
    pub menu_id: uuid::Uuid,
    /// 创建时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ==================== 数据访问 ====================

/// 角色分页查询条件。
///
/// 由处理器完成参数解析后传入；模糊匹配字段保持原始关键词，由本层统一转换为
/// `LIKE` 模式，避免查询细节泄漏到处理器。
pub(crate) struct RolePageFilter {
    /// 角色名称（模糊匹配）。
    pub role_name: Option<String>,
    /// 角色编码（模糊匹配）。
    pub role_code: Option<String>,
    /// 角色状态（精确匹配）。
    pub status: Option<i16>,
}

/// 分页查询角色（支持角色名 / 编码模糊与状态过滤）。
///
/// # 参数
///
/// - `filter`：过滤条件。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `sort_order` 升序、`created_at` 倒序的角色分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_roles(
    filter: &RolePageFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysRoleModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_role WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.role_name.as_deref()) {
        sql.push_str(" AND role_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.role_code.as_deref()) {
        sql.push_str(" AND role_code LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    sql.push_str(" ORDER BY sort_order ASC, created_at DESC");

    let Page {
        list,
        total,
        page,
        size,
        pages,
    } = db()
        .paginate_with_params(&sql, params, page, page_size)
        .await?;
    let list = list
        .into_iter()
        .map(SysRoleModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 查询全部启用角色（按排序值升序，供下拉选择）。
///
/// # 返回值
///
/// 启用角色列表。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn list_enabled_roles() -> std::result::Result<Vec<SysRoleModel>, DbError> {
    SysRoleModel::find_by(
        db(),
        "status = ? ORDER BY sort_order ASC",
        (TypeMapper::to_value(&constants::STATUS_ENABLED),),
    )
    .await
}

/// 分页查询持有指定角色的用户。
///
/// # 参数
///
/// - `role_id`：角色 ID。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的用户分页结果（用户表与用户-角色关联表内连接）。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_role_users(
    role_id: &uuid::Uuid,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysUserModel>, DbError> {
    let sql = "SELECT u.* FROM sys_user u \
         INNER JOIN sys_user_role ur ON ur.user_id = u.id \
         WHERE ur.role_id = ? AND u.deleted_at IS NULL \
         ORDER BY u.created_at DESC";
    let params: Vec<Value> = vec![Value::from(role_id)];

    let Page {
        list,
        total,
        page,
        size,
        pages,
    } = db()
        .paginate_with_params(sql, params, page, page_size)
        .await?;
    let list = list
        .into_iter()
        .map(SysUserModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 统计占用指定角色编码的未删除角色数。
///
/// # 参数
///
/// - `role_code`：角色编码。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 匹配的角色数量。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_roles_by_code(
    role_code: &str,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude {
        Some(id) => {
            SysRoleModel::count_by(
                db(),
                "role_code = ? AND id <> ?",
                (Value::from(role_code), &id),
            )
            .await
        }
        None => SysRoleModel::count_by(db(), "role_code", (Value::from(role_code),)).await,
    }
}

/// 重建角色菜单授权：在单个事务内先清空全部绑定，再按给定菜单 ID 逐条写入。
///
/// 清空与重建处于同一事务，任一步失败整体回滚，不会出现「旧授权已清空而新授权
/// 未写完」的中间态。
///
/// # 参数
///
/// - `role_id`：角色 ID。
/// - `menu_ids`：已校验的菜单 ID 列表；空切片表示清空全部授权。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]（事务回滚）。
pub(crate) async fn replace_role_menus(
    role_id: &uuid::Uuid,
    menu_ids: &[uuid::Uuid],
) -> std::result::Result<(), DbError> {
    let now = chrono::Utc::now();
    db().transaction(|tx| async move {
        // 关联表为硬删表：重新授权即整批重建
        SysRoleMenuModel::delete_by(&tx, "role_id", (role_id,)).await?;
        for menu_id in menu_ids {
            // 关联表关闭时间戳 Hook，created_at 需显式赋值
            let mut binding = SysRoleMenuModel {
                id: uuid::Uuid::now_v7(),
                role_id: *role_id,
                menu_id: *menu_id,
                created_at: now,
            };
            binding.insert(&tx).await?;
        }
        Ok(())
    })
    .await
}

// ==================== 启用角色下拉缓存 ====================

/// 读取缓存的启用角色下拉列表。
///
/// 命中返回 `Some`，未命中或缓存不可用返回 `None`（调用方回退查库并回填）。
pub(crate) async fn load_cached_enabled_roles() -> Option<Vec<crate::system::role::dto::RoleRes>> {
    let store = try_cache()?;
    Cache::get(store, constants::CACHE_KEY_ROLE_ENABLED)
        .await
        .ok()
        .flatten()
}

/// 写入启用角色下拉列表缓存；失败只告警，不影响主流程。
pub(crate) async fn cache_enabled_roles(roles: &[crate::system::role::dto::RoleRes]) {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::set_ex(
        store,
        constants::CACHE_KEY_ROLE_ENABLED,
        roles,
        constants::CACHE_HOT_QUERY_TTL_SECS,
    )
    .await
    {
        tracing::warn!(error = %e, "写入启用角色缓存失败");
    }
}

/// 失效启用角色下拉缓存（角色写操作 / 授权变更后调用）。
pub(crate) async fn invalidate_enabled_roles_cache() {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::del(store, constants::CACHE_KEY_ROLE_ENABLED).await {
        tracing::warn!(error = %e, "失效启用角色缓存失败");
    }
}
