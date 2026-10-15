//! 部门机构的强类型模型。
//!
//! 对应迁移 `V0006__create_sys_dept`。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（树查询、详情、增删改、唯一性校验）与领域行为集中在本文件；
//! 仅调用框架快捷方法（`find_or_none` / `count_by` / `insert` / `update` /
//! `delete` 等）的简单读写由 `handler.rs` 直接执行。

use alun::prelude::*;

use crate::shared::constants::{self, EnableStatus};
use crate::utils;

/// `sys_dept` 表对应的强类型模型（部门机构）。
///
/// # 树形结构与冗余路径
///
/// `parent_id` 为 `NULL` 时是根部门；`dept_path` 冗余保存层级路径
/// （形如 `/{根ID}/{子ID}/`），用于「本部门及子部门」的范围查询，避免递归。
/// 该字段 v1 预留，P3 后可启用。
///
/// # 弱引用
///
/// `leader_id`（负责人）与 `create_by` / `update_by` 均为对 `sys_user` 的弱引用，
/// 不建外键，避免删除用户时阻塞。
#[derive(Debug, Clone, Model)]
#[table("sys_dept")]
pub struct SysDeptModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 部门编码，软删表上的部分唯一列。
    pub dept_code: String,
    /// 部门名称。
    pub dept_name: String,
    /// 父部门 ID（`NULL` 为根部门）。
    pub parent_id: Option<uuid::Uuid>,
    /// 层级路径（`/{根ID}/{子ID}/`），用于子部门范围查询。
    pub dept_path: Option<String>,
    /// 负责人用户 ID（弱引用 `sys_user`）。
    pub leader_id: Option<uuid::Uuid>,
    /// 联系电话。
    pub phone: Option<String>,
    /// 部门邮箱。
    pub email: Option<String>,
    /// 部门状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 排序号，升序展示。
    pub sort_order: i32,
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

// ==================== 数据访问 ====================

/// 部门树查询条件。
///
/// 由处理器完成参数解析后传入；模糊匹配字段保持原始关键词，由本层统一转换为
/// `LIKE` 模式。
pub(crate) struct DeptTreeFilter {
    /// 部门名称（模糊匹配）。
    pub dept_name: Option<String>,
    /// 部门状态（精确匹配）。
    pub status: Option<i16>,
}

/// 按条件查询部门（扁平列表，供上层组装为树）。
///
/// # 参数
///
/// - `filter`：过滤条件（部门名模糊与状态过滤，均可选）。
///
/// # 返回值
///
/// 已按 `sort_order`、`created_at` 升序的扁平部门列表。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn find_dept_list(
    filter: &DeptTreeFilter,
) -> std::result::Result<Vec<SysDeptModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_dept WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();

    if let Some(pattern) = utils::like_pattern(filter.dept_name.as_deref()) {
        sql.push_str(" AND dept_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    sql.push_str(" ORDER BY sort_order ASC, created_at ASC");

    let rows = db().find_with_params(&sql, params).await?;
    rows.into_iter()
        .map(SysDeptModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()
}

/// 批量查询用户显示名（`real_name` 优先，为空回退 `username`）。
///
/// 供部门树填充 `leader_name`：一次 `IN` 查询取回全部负责人，避免逐部门 N+1。
///
/// # 参数
///
/// - `ids`：用户 ID 列表（内部自动容忍重复，无需调用方去重）。
///
/// # 返回值
///
/// 「用户 ID → 显示名」映射；已删除用户不会出现在结果中。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn find_user_display_names(
    ids: &[uuid::Uuid],
) -> std::result::Result<std::collections::HashMap<uuid::Uuid, String>, DbError> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = format!(
        "SELECT id, COALESCE(NULLIF(real_name, ''), username) AS display_name \
         FROM sys_user \
         WHERE id IN ({placeholders}) AND deleted_at IS NULL"
    );
    let params: Vec<Value> = ids.iter().map(|id| Value::from(*id)).collect();
    let rows = db().find_with_params(&sql, params).await?;
    let mut map = std::collections::HashMap::with_capacity(rows.len());
    for row in rows {
        if let (Ok(id), Ok(name)) = (
            row.get_typed::<uuid::Uuid>("id"),
            row.get_typed::<String>("display_name"),
        ) {
            map.insert(id, name);
        }
    }
    Ok(map)
}

/// 统计占用指定部门编码的未删除部门数。
///
/// # 参数
///
/// - `dept_code`：部门编码。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 匹配的部门数量。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_depts_by_code(
    dept_code: &str,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude {
        Some(id) => {
            SysDeptModel::count_by(
                db(),
                "dept_code = ? AND id <> ?",
                (Value::from(dept_code), &id),
            )
            .await
        }
        None => SysDeptModel::count_by(db(), "dept_code", (Value::from(dept_code),)).await,
    }
}

/// 统计同一上级部门下同名部门的数量（同级名称唯一性校验）。
///
/// 仅在 `parent_id` 相同的兄弟部门间判重：不同层级允许出现同名部门。
///
/// # 参数
///
/// - `dept_name`：待校验的部门名称。
/// - `parent_id`：上级部门 ID；`None` 表示根部门（按 `parent_id IS NULL` 比较）。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 同级同名的部门数量。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_depts_by_name(
    dept_name: &str,
    parent_id: Option<uuid::Uuid>,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match (parent_id, exclude) {
        (Some(parent), Some(exclude)) => {
            SysDeptModel::count_by(
                db(),
                "dept_name = ? AND parent_id = ? AND id <> ?",
                (
                    Value::from(dept_name),
                    Value::from(parent),
                    Value::from(exclude),
                ),
            )
            .await
        }
        (Some(parent), None) => {
            SysDeptModel::count_by(
                db(),
                "dept_name = ? AND parent_id = ?",
                (Value::from(dept_name), Value::from(parent)),
            )
            .await
        }
        (None, Some(exclude)) => {
            SysDeptModel::count_by(
                db(),
                "dept_name = ? AND parent_id IS NULL AND id <> ?",
                (Value::from(dept_name), Value::from(exclude)),
            )
            .await
        }
        (None, None) => {
            SysDeptModel::count_by(
                db(),
                "dept_name = ? AND parent_id IS NULL",
                (Value::from(dept_name),),
            )
            .await
        }
    }
}

/// 查询某部门及其全部子孙部门的 ID（递归 CTE，含自身）。
///
/// 用于「是否包含子部门用户」筛选：不依赖可能因搬迁而未同步的 `dept_path`，
/// 直接按 `parent_id` 递归取整棵子树，结果始终与当前层级一致。
///
/// # 参数
///
/// - `root_id`：根部门 ID。
///
/// # 返回值
///
/// 根部门及其所有后代部门的 ID 列表；根部门不存在时仅为空。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn find_dept_subtree_ids(
    root_id: &uuid::Uuid,
) -> std::result::Result<Vec<uuid::Uuid>, DbError> {
    let sql = "WITH RECURSIVE dept_tree AS ( \
         SELECT id FROM sys_dept WHERE id = ? AND deleted_at IS NULL \
         UNION ALL \
         SELECT c.id FROM sys_dept c \
         INNER JOIN dept_tree t ON c.parent_id = t.id \
         WHERE c.deleted_at IS NULL \
       ) SELECT id FROM dept_tree";
    let rows = db()
        .find_with_params(sql, vec![Value::from(root_id)])
        .await?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        ids.push(row.get_typed::<uuid::Uuid>("id")?);
    }
    Ok(ids)
}

// ==================== 部门树缓存 ====================

/// 读取缓存的全量部门树。
///
/// 仅缓存「无任何过滤条件」的全量树；命中返回 `Some`，未命中或缓存不可用返回 `None`
/// （由调用方回退查库并回填）。
pub(crate) async fn load_cached_tree() -> Option<Vec<crate::system::dept::dto::DeptNode>> {
    let store = try_cache()?;
    Cache::get(store, constants::CACHE_KEY_DEPT_TREE)
        .await
        .ok()
        .flatten()
}

/// 写入全量部门树缓存。
///
/// 缓存失败只记告警，不影响主流程（缓存仅为读优化）。
pub(crate) async fn cache_tree(nodes: &[crate::system::dept::dto::DeptNode]) {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::set_ex(
        store,
        constants::CACHE_KEY_DEPT_TREE,
        nodes,
        constants::CACHE_HOT_QUERY_TTL_SECS,
    )
    .await
    {
        tracing::warn!(error = %e, "写入部门树缓存失败");
    }
}

/// 失效全量部门树缓存（部门写操作后调用）。
pub(crate) async fn invalidate_tree_cache() {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::del(store, constants::CACHE_KEY_DEPT_TREE).await {
        tracing::warn!(error = %e, "失效部门树缓存失败");
    }
}
