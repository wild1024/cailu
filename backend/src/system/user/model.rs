//! 用户与「用户-角色」关联的强类型模型。
//!
//! 对应迁移 `V0001__create_sys_user` 与 `V0003__create_sys_user_role`。
//!
//! 按项目分层约定：**需要编写 SQL 的查询与领域行为集中在本文件**；仅调用框架
//! 快捷方法（`find_or_none` / `find_by` / `count_by` / `of(...).set(...).update` /
//! `insert` / `update` / `delete` 等）的简单读写由 `handler.rs` 直接执行，
//! 不再在模型层重复封装一层。
//!
//! - [`SysUserModel`]：软删表，含 `created_at` / `updated_at`，框架自动挂载
//!   `TimestampHook` 并自动过滤 `deleted_at`；
//! - [`SysUserRoleModel`]：关联表，仅含 `created_at`，采用 **硬删 + 关闭时间戳
//!   Hook**（`#[model(no_timestamps, hard_delete)]`），重新授权即重建记录。

use alun::prelude::*;

use crate::shared::constants::EnableStatus;
use crate::utils;

/// `sys_user` 表对应的强类型模型（用户）。
///
/// # 主键
///
/// `id: Uuid` 由应用侧写入 `Uuid::now_v7()`（时间有序，避免 B-tree 随机插入），
/// 数据库的 `DEFAULT gen_random_uuid()` 仅作兜底。
///
/// # 软删与唯一性
///
/// `deleted_at` 为软删标记；`username` 的部分唯一索引带 `WHERE deleted_at IS NULL`，
/// 因此删除后可重建同名账号。
///
/// # 字段类型说明
///
/// - `status` 用 [`EnableStatus`] 枚举（`SMALLINT` 整数判别值编码）；
/// - `login_fail_count` 对应 `SMALLINT`，用 `i16` 承载；
/// - `lock_time` / `last_login_at` / 审计列为可空时间或可空 UUID。
#[derive(Debug, Clone, Model)]
#[table("sys_user")]
pub struct SysUserModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 登录账号，软删表上的部分唯一列。
    pub username: String,
    /// 加密后的密码哈希（Argon2 / BCrypt），严禁存明文。
    pub password: String,
    /// 真实姓名。
    pub real_name: Option<String>,
    /// 电子邮箱。
    pub email: Option<String>,
    /// 手机号码。
    pub mobile: Option<String>,
    /// 所属部门 ID（弱引用 `sys_dept`，不建外键）。
    pub dept_id: Option<uuid::Uuid>,
    /// 账号状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 连续登录失败次数，达到阈值后锁定账号。
    pub login_fail_count: i16,
    /// 是否已锁定（锁定后禁止登录）。
    pub is_locked: bool,
    /// 锁定时间（未锁定为 `NULL`）。
    pub lock_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后登录时间（从未登录为 `NULL`）。
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
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

/// `sys_user_role` 表对应的强类型模型（用户-角色关联）。
///
/// # 硬删关联表
///
/// 表上仅有 `created_at`（无 `updated_at`），且唯一约束 `(user_id, role_id)`，
/// 因此派生标注 `#[model(no_timestamps, hard_delete)]`：
///
/// - `no_timestamps`：不自动挂载 `TimestampHook`，`created_at` 由写入方显式赋值；
/// - `hard_delete`：`delete` 走物理删除（重新授权即重建记录，不保留历史行）。
///
/// # 过期语义
///
/// `expire_time` 为 `NULL` 表示永久有效；非空表示该角色在指定时刻后失效
/// （过期判定由认证逻辑负责，不在模型层处理）。
#[derive(Debug, Clone, Model)]
#[table("sys_user_role")]
#[model(no_timestamps, hard_delete)]
pub struct SysUserRoleModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 用户 ID，引用 `sys_user(id)`。
    pub user_id: uuid::Uuid,
    /// 角色 ID，引用 `sys_role(id)`。
    pub role_id: uuid::Uuid,
    /// 角色过期时间（`NULL` 表示永久）。
    pub expire_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 创建时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ==================== 数据访问函数 ====================

/// 用户分页查询条件。
///
/// 由处理器完成参数校验（含部门 ID 解析、子树展开）后传入；模糊匹配字段保持原始
/// 关键词，由本层统一转换为 `LIKE` 模式，避免查询细节泄漏到处理器。
pub(crate) struct UserPageFilter {
    /// 用户名关键词（模糊匹配）。
    pub username: Option<String>,
    /// 真实姓名关键词（模糊匹配）。
    pub real_name: Option<String>,
    /// 邮箱关键词（模糊匹配）。
    pub email: Option<String>,
    /// 手机号关键词（模糊匹配）。
    pub mobile: Option<String>,
    /// 账号状态（精确匹配，0 停用 / 1 正常）。
    pub status: Option<i16>,
    /// 部门 ID 集合（精确匹配，含子部门时为整棵子树；`None` 表示不按部门过滤）。
    pub dept_ids: Option<Vec<uuid::Uuid>>,
    /// 创建时间起始（含）。
    pub create_begin: Option<chrono::DateTime<chrono::Utc>>,
    /// 创建时间截止（含）。
    pub create_end: Option<chrono::DateTime<chrono::Utc>>,
}

/// 用户列表行（`sys_user` LEFT JOIN `sys_dept`）。
pub(crate) struct UserWithDept {
    /// 用户模型。
    pub user: SysUserModel,
    /// 所属部门名称（无部门或部门已删除时为空串）。
    pub dept_name: String,
}

/// 用户关联角色的查询行（`sys_user_role` JOIN `sys_role`）。
pub(crate) struct UserRoleRow {
    /// 用户 ID。
    pub user_id: uuid::Uuid,
    /// 角色 ID。
    pub role_id: uuid::Uuid,
    /// 角色编码。
    pub role_code: String,
    /// 角色名称。
    pub role_name: String,
}

/// 分页查询用户（支持用户名 / 姓名 / 邮箱 / 手机号模糊、状态、部门子树与创建时间范围过滤）。
///
/// # 参数
///
/// - `filter`：分页与过滤条件。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的用户分页结果，列表项附带部门名。
///
/// # 异常情况
///
/// 数据库异常、行缺少 `dept_name` 列或行映射失败返回 [`DbError`]。
pub(crate) async fn page_users(
    filter: &UserPageFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<UserWithDept>, DbError> {
    let mut sql = String::from(
        "SELECT u.*, d.dept_name AS dept_name FROM sys_user u \
         LEFT JOIN sys_dept d ON d.id = u.dept_id AND d.deleted_at IS NULL \
         WHERE u.deleted_at IS NULL",
    );
    let mut params: Vec<Value> = Vec::new();

    if let Some(pattern) = utils::like_pattern(filter.username.as_deref()) {
        sql.push_str(" AND u.username LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.real_name.as_deref()) {
        sql.push_str(" AND u.real_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.email.as_deref()) {
        sql.push_str(" AND u.email LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.mobile.as_deref()) {
        sql.push_str(" AND u.mobile LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND u.status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    if let Some(dept_ids) = &filter.dept_ids {
        if dept_ids.is_empty() {
            // 选定了部门但子树为空（理论上至少含自身），兜底返回空集
            sql.push_str(" AND 1 = 0");
        } else {
            let placeholders = dept_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            sql.push_str(&format!(" AND u.dept_id IN ({placeholders})"));
            for id in dept_ids {
                params.push(Value::from(id));
            }
        }
    }
    if let Some(begin) = filter.create_begin {
        sql.push_str(" AND u.created_at >= ?");
        params.push(Value::from(begin));
    }
    if let Some(end) = filter.create_end {
        sql.push_str(" AND u.created_at <= ?");
        params.push(Value::from(end));
    }
    sql.push_str(" ORDER BY u.created_at DESC");

    let Page {
        list,
        total,
        page,
        size,
        pages,
    } = db()
        .paginate_with_params(&sql, params, page, page_size)
        .await?;

    let mut users: Vec<UserWithDept> = Vec::with_capacity(list.len());
    for row in list {
        let dept_name = row
            .get_typed::<Option<String>>("dept_name")?
            .unwrap_or_default();
        let user = SysUserModel::from_row(row)?;
        users.push(UserWithDept { user, dept_name });
    }
    Ok(Page {
        list: users,
        total,
        page,
        size,
        pages,
    })
}

/// 批量查询一批用户关联的角色（单次 IN 查询，避免列表场景 N+1）。
///
/// # 参数
///
/// - `user_ids`：用户 ID 列表；空切片直接返回空。
///
/// # 返回值
///
/// 用户-角色关联行（含角色编码与名称），按角色排序值升序；调用方按 `user_id` 聚合。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn find_user_roles_for_users(
    user_ids: &[uuid::Uuid],
) -> std::result::Result<Vec<UserRoleRow>, DbError> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT ur.user_id AS user_id, r.id AS role_id, r.role_code AS role_code, \
         r.role_name AS role_name FROM sys_user_role ur \
         INNER JOIN sys_role r ON r.id = ur.role_id AND r.deleted_at IS NULL \
         WHERE ur.user_id IN ({placeholders}) ORDER BY r.sort_order ASC"
    );
    let params: Vec<Value> = user_ids.iter().map(Value::from).collect();
    let rows = db().find_with_params(&sql, params).await?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(UserRoleRow {
            user_id: row.get_typed::<uuid::Uuid>("user_id")?,
            role_id: row.get_typed::<uuid::Uuid>("role_id")?,
            role_code: row.get_typed::<String>("role_code")?,
            role_name: row.get_typed::<String>("role_name")?,
        });
    }
    Ok(result)
}

/// 统计占用指定登录账号的用户数（软删表上仅统计未删除行）。
///
/// # 参数
///
/// - `username`：待校验的登录账号。
/// - `exclude`：更新场景需排除的自身 ID；新建场景传 `None`。
///
/// # 返回值
///
/// 匹配的用户行数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_users_by_username(
    username: &str,
    exclude: Option<uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude {
        Some(id) => {
            SysUserModel::count_by(
                db(),
                "username = ? AND id <> ?",
                (Value::from(username), Value::from(&id)),
            )
            .await
        }
        None => SysUserModel::count_by(db(), "username", (Value::from(username),)).await,
    }
}

/// 为用户批量写入角色绑定（不校验角色存在性，调用方负责校验）。
///
/// 接受任意 [`SqlExecutor`]，因此既可在全局连接上执行，也可在事务内执行，
/// 从而与用户创建 / 角色重建处于同一事务。
///
/// # 参数
///
/// - `ex`：数据库执行器（连接或事务）。
/// - `user_id`：用户 ID。
/// - `role_ids`：角色 ID 列表（空切片直接返回）。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn insert_user_roles(
    ex: &impl SqlExecutor,
    user_id: &uuid::Uuid,
    role_ids: &[uuid::Uuid],
) -> std::result::Result<(), DbError> {
    let now = chrono::Utc::now();
    for role_id in role_ids {
        // 关联表关闭时间戳 Hook，created_at 需显式赋值
        let mut binding = SysUserRoleModel {
            id: uuid::Uuid::now_v7(),
            user_id: *user_id,
            role_id: *role_id,
            expire_time: None,
            created_at: now,
        };
        binding.insert(ex).await?;
    }
    Ok(())
}

/// 重建用户角色：在给定执行器上先清空全部绑定，再按给定角色 ID 写入。
///
/// 通常置于事务中执行，保证「清空 + 重建」原子完成。
///
/// # 参数
///
/// - `ex`：数据库执行器（建议为事务）。
/// - `user_id`：用户 ID。
/// - `role_ids`：角色 ID 列表；空切片表示清空全部角色。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn replace_user_roles(
    ex: &impl SqlExecutor,
    user_id: &uuid::Uuid,
    role_ids: &[uuid::Uuid],
) -> std::result::Result<(), DbError> {
    // 关联表为硬删表：重新授权即整批重建
    SysUserRoleModel::delete_by(ex, "user_id", (user_id,)).await?;
    insert_user_roles(ex, user_id, role_ids).await
}

/// 批量把一批用户调入指定部门（单条 `UPDATE ... WHERE id IN (...)`）。
///
/// 仅在事务内调用（`execute_with_params` 为连接 / 事务的固有写方法）。
///
/// # 参数
///
/// - `tx`：事务执行器。
/// - `user_ids`：待调整的用户 ID 列表（调用方保证非空）。
/// - `dept_id`：目标部门 ID。
/// - `operator`：操作人 ID（写入 `update_by`，可为 `None`）。
///
/// # 返回值
///
/// 受影响行数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn batch_update_dept(
    tx: &Transaction,
    user_ids: &[uuid::Uuid],
    dept_id: &uuid::Uuid,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<u64, DbError> {
    let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "UPDATE sys_user SET dept_id = ?, updated_at = now(), update_by = ? \
         WHERE id IN ({placeholders}) AND deleted_at IS NULL"
    );
    let mut params: Vec<Value> = vec![Value::from(dept_id)];
    match operator {
        Some(operator) => params.push(Value::from(operator)),
        None => params.push(Value::Null),
    }
    for id in user_ids {
        params.push(Value::from(id));
    }
    tx.execute_with_params(&sql, params).await
}
