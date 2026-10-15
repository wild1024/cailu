//! 操作日志的强类型模型。
//!
//! 对应迁移 `V0011__create_operation_logs`。本表为**追加写**表：只有 `created_at`
//! 承载操作时间（无 `updated_at`），且采用物理删除，故派生标注
//! `#[model(no_timestamps, hard_delete)]`。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（分页、按时间清理）集中在**本文件**；仅调用框架快捷方法
//! （`find_or_none` 等）的简单读写由 `handler.rs` 直接执行。
//! `delete_operation_logs_before` 同时被 `task`（定时清理）与 `handler` 复用，
//! 保留在模型层。

use alun::prelude::*;

use crate::utils;

/// `operation_logs` 表对应的强类型模型（操作日志）。
///
/// # 追加写表
///
/// - `no_timestamps`：不自动挂载 `TimestampHook`，`created_at` 即「操作发生时刻」，
///   由写入方（中间件）显式赋值，不能被写入时刻覆盖；
/// - `hard_delete`：无 `deleted_at` 列，`delete` 走物理删除，日志按时间清理。
///
/// # JSON 字段
///
/// `detail` / `request_params` / `response_data` 为可空 `JSONB`，用
/// `#[json(nullable)]` 标注，模型字段为 `Option<JsonValue>`。其中请求体与响应体
/// 由操作日志中间件**脱敏后**捕获写入（敏感字段见
/// [`crate::middleware::operation_log`]），非 JSON 或超限载荷不落库。
///
/// # IP 字段
///
/// `ip` 列为 PostgreSQL `INET`。`alun` 已启用 `ipnetwork` feature，框架为
/// `std::net::IpAddr` 实现了 `TypeMapper`（映射到 `Type::Inet`），因此模型侧
/// 直接以 `Option<std::net::IpAddr>` 承载，无需手工文本转换。
#[derive(Debug, Clone, Model)]
#[table("operation_logs")]
#[model(no_timestamps, hard_delete)]
pub struct OperationLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 请求追踪 ID（与响应头 / 日志中的 request_id 一致）。
    pub request_id: String,
    /// 操作用户 ID（匿名请求为 `NULL`）。
    pub user_id: Option<uuid::Uuid>,
    /// 操作用户名（冗余，避免查询时联表）。
    pub username: Option<String>,
    /// 操作类型（如 `USER_CREATE`）。
    pub operation: String,
    /// 操作对象类型。
    pub target_type: Option<String>,
    /// 操作对象 ID。
    pub target_id: Option<String>,
    /// 操作详情（已脱敏的 JSONB）。
    #[json(nullable)]
    pub detail: Option<JsonValue>,
    /// 客户端 IP（对应 `INET` 列，见结构体文档「IP 字段」）。
    pub ip: Option<std::net::IpAddr>,
    /// User-Agent。
    pub user_agent: Option<String>,
    /// 处理耗时（毫秒）。
    pub duration_ms: Option<i32>,
    /// 请求路径。
    pub request_path: Option<String>,
    /// 请求方法。
    pub request_method: Option<String>,
    /// 请求参数（脱敏后的 JSONB）。
    #[json(nullable)]
    pub request_params: Option<JsonValue>,
    /// 响应数据（脱敏后的结构化快照，JSONB）。
    #[json(nullable)]
    pub response_data: Option<JsonValue>,
    /// HTTP 状态码。
    pub status_code: Option<i32>,
    /// 操作时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `audit_logs` 表对应的强类型模型（审计日志）。
///
/// # 追加写表
///
/// 与 [`OperationLogModel`] 同样标注 `#[model(no_timestamps, hard_delete)]`：
/// 只写不改删，按时间清理。`before_data` / `after_data` / `detail` 为可空 JSONB。
#[derive(Debug, Clone, Model)]
#[table("audit_logs")]
#[model(no_timestamps, hard_delete)]
pub struct AuditLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 请求追踪 ID。
    pub request_id: String,
    /// 操作用户 ID。
    pub user_id: Option<uuid::Uuid>,
    /// 操作用户名（冗余）。
    pub username: Option<String>,
    /// 操作类型（如 `USER_CREATE`）。
    pub operation: String,
    /// 操作对象类型。
    pub target_type: Option<String>,
    /// 操作对象 ID。
    pub target_id: Option<String>,
    /// 变更前数据快照。
    #[json(nullable)]
    pub before_data: Option<JsonValue>,
    /// 变更后数据快照。
    #[json(nullable)]
    pub after_data: Option<JsonValue>,
    /// 补充详情。
    #[json(nullable)]
    pub detail: Option<JsonValue>,
    /// 客户端 IP（`INET` 列）。
    pub ip: Option<std::net::IpAddr>,
    /// User-Agent。
    pub user_agent: Option<String>,
    /// 审计时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `auth_logs` 表对应的强类型模型（认证日志）。
///
/// # 追加写表
///
/// 记录登录 / 登出 / 刷新令牌的认证结果；`device_type` / `browser_type` /
/// `os_type` 由 `parse_user_agent` 从 User-Agent 解析后写入。
#[derive(Debug, Clone, Model)]
#[table("auth_logs")]
#[model(no_timestamps, hard_delete)]
pub struct AuthLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 请求追踪 ID。
    pub request_id: String,
    /// 登录用户 ID（用户不存在时为空）。
    pub user_id: Option<uuid::Uuid>,
    /// 登录用户名。
    pub username: Option<String>,
    /// 认证类型：`LOGIN` / `LOGOUT` / `REFRESH_TOKEN`。
    pub auth_type: String,
    /// 认证状态：`SUCCESS` / `FAIL`。
    pub auth_status: String,
    /// 客户端 IP（`INET` 列）。
    pub ip: Option<std::net::IpAddr>,
    /// 登录地点（由 IP 解析，未启用解析时为空）。
    pub location: Option<String>,
    /// 设备类型：`PC` / `MOBILE` / `TABLET`。
    pub device_type: Option<String>,
    /// 浏览器类型。
    pub browser_type: Option<String>,
    /// 操作系统类型。
    pub os_type: Option<String>,
    /// 完整 User-Agent。
    pub user_agent: Option<String>,
    /// 失败原因（成功时为空）。
    pub error_message: Option<String>,
    /// 登录耗时（毫秒）。
    pub login_duration: Option<i32>,
    /// 认证时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `security_logs` 表对应的强类型模型（安全日志）。
///
/// # 追加写表
///
/// 登录失败、越权、可疑注入等安全事件的落点；`detail` 为脱敏后的 JSONB。
#[derive(Debug, Clone, Model)]
#[table("security_logs")]
#[model(no_timestamps, hard_delete)]
pub struct SecurityLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 请求追踪 ID（非请求触发时为空）。
    pub request_id: Option<String>,
    /// 关联用户 ID（可能为空）。
    pub user_id: Option<uuid::Uuid>,
    /// 用户名（尝试登录或操作的用户名）。
    pub username: Option<String>,
    /// 事件类型：`LOGIN_FAIL` / `PERMISSION_DENY` / `SQL_INJECT_ATTEMPT` 等。
    pub event_type: String,
    /// 事件详情（脱敏 JSON）。
    #[json(nullable)]
    pub detail: Option<JsonValue>,
    /// 客户端 IP（`INET` 列）。
    pub ip: Option<std::net::IpAddr>,
    /// User-Agent。
    pub user_agent: Option<String>,
    /// 事件时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `password_change_logs` 表对应的强类型模型（密码变更日志）。
///
/// # 追加写表
///
/// 记录重置 / 修改 / 强制改密的操作者与结果；`change_type` 取值
/// `RESET` / `UPDATE` / `FORCED`，`operator_type` 取值 `SELF` / `ADMIN` / `SYSTEM`。
#[derive(Debug, Clone, Model)]
#[table("password_change_logs")]
#[model(no_timestamps, hard_delete)]
pub struct PasswordChangeLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 请求追踪 ID。
    pub request_id: String,
    /// 被修改密码的用户 ID。
    pub user_id: uuid::Uuid,
    /// 被修改密码的用户名。
    pub username: String,
    /// 变更类型：`RESET` / `UPDATE` / `FORCED`。
    pub change_type: String,
    /// 操作者类型：`SELF` / `ADMIN` / `SYSTEM`。
    pub operator_type: String,
    /// 操作者 ID（本人修改时等于 `user_id`）。
    pub operator_id: Option<uuid::Uuid>,
    /// 客户端 IP（`INET` 列）。
    pub ip: Option<std::net::IpAddr>,
    /// User-Agent。
    pub user_agent: Option<String>,
    /// 变更状态：`SUCCESS` / `FAIL`。
    pub change_status: String,
    /// 失败原因。
    pub error_message: Option<String>,
    /// 操作时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ==================== 查询条件与访问函数 ====================

/// 操作日志分页查询条件。
///
/// 由处理器完成参数校验（含时间格式解析）后传入；模糊匹配字段保持原始关键词，
/// 由本层统一转换为 `LIKE` 模式，避免查询细节泄漏到处理器。
pub(crate) struct OperationLogFilter {
    /// 操作用户名（模糊匹配）。
    pub username: Option<String>,
    /// 操作类型（模糊匹配）。
    pub operation: Option<String>,
    /// 请求路径（模糊匹配）。
    pub request_path: Option<String>,
    /// HTTP 状态码（精确匹配）。
    pub status_code: Option<i32>,
    /// 起始时间（含）。
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间（不含）。
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// 支持 `to_jsonb` 泛化查询的日志表。
///
/// 用枚举而非字符串承载表名，**从类型层面杜绝标识符注入**：表名只可能取自
/// 编译期固定的几个变体，调用方无法传入任意字符串。
pub(crate) enum JsonLogTable {
    /// 审计日志表。
    Audit,
    /// 认证日志表。
    Auth,
    /// 安全日志表。
    Security,
    /// 密码变更日志表。
    PasswordChange,
}

impl JsonLogTable {
    /// 返回对应的物理表名。
    fn name(self) -> &'static str {
        match self {
            JsonLogTable::Audit => "audit_logs",
            JsonLogTable::Auth => "auth_logs",
            JsonLogTable::Security => "security_logs",
            JsonLogTable::PasswordChange => "password_change_logs",
        }
    }
}

/// 分页查询操作日志。
///
/// # 参数
///
/// - `filter`：过滤条件（模糊 / 精确 / 时间区间）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的操作日志分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_operation_logs(
    filter: &OperationLogFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<OperationLogModel>, DbError> {
    let mut sql = String::from("SELECT * FROM operation_logs WHERE 1=1");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.username.as_deref()) {
        sql.push_str(" AND username LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.operation.as_deref()) {
        sql.push_str(" AND operation LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.request_path.as_deref()) {
        sql.push_str(" AND request_path LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status_code) = filter.status_code {
        sql.push_str(" AND status_code = ?");
        params.push(TypeMapper::to_value(&status_code));
    }
    if let Some(start) = filter.start_time {
        sql.push_str(" AND created_at >= ?");
        params.push(TypeMapper::to_value(&start));
    }
    if let Some(end) = filter.end_time {
        sql.push_str(" AND created_at < ?");
        params.push(TypeMapper::to_value(&end));
    }
    sql.push_str(" ORDER BY created_at DESC");

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
        .map(OperationLogModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 物理删除早于指定时刻的操作日志。
///
/// # 参数
///
/// - `cutoff`：时间阈值，删除 `created_at < cutoff` 的行。
///
/// # 返回值
///
/// 实际删除的行数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn delete_operation_logs_before(
    cutoff: chrono::DateTime<chrono::Utc>,
) -> std::result::Result<u64, DbError> {
    OperationLogModel::delete_by(db(), "created_at < ?", (TypeMapper::to_value(&cutoff),)).await
}

/// 物理删除某张 JSON 日志表中早于指定时刻的记录。
///
/// 供审计 / 认证 / 安全 / 密码变更四张表的清理任务复用；表名取自
/// [`JsonLogTable`] 枚举，不接受外部输入。
///
/// # 参数
///
/// - `table`：目标日志表。
/// - `cutoff`：时间阈值，删除 `created_at < cutoff` 的行。
///
/// # 返回值
///
/// 实际删除的行数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn delete_json_logs_before(
    table: JsonLogTable,
    cutoff: chrono::DateTime<chrono::Utc>,
) -> std::result::Result<u64, DbError> {
    let sql = format!("DELETE FROM {} WHERE created_at < ?", table.name());
    db().execute_with_params(&sql, vec![TypeMapper::to_value(&cutoff)])
        .await
}

/// 泛化分页查询单表日志（审计 / 认证 / 安全 / 密码变更四张表共用）。
///
/// 以 `to_jsonb(t)` 把整行序列化为单个 JSONB 列返回，避免为四张表各写一套列映射；
/// 过滤条件只保留四表共有的 `username` 与时间区间。
///
/// # 参数
///
/// - `table`：日志表枚举（编译期固定，杜绝标识符注入）。
/// - `username`：用户名关键词（模糊匹配）。
/// - `start_time`：起始时间（含）。
/// - `end_time`：结束时间（不含）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的 JSON 行分页结果。
///
/// # 异常情况
///
/// 数据库异常、行缺少 `data` 列或 JSON 解析失败返回 [`DbError`]。
pub(crate) async fn page_json_logs(
    table: JsonLogTable,
    username: Option<String>,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<JsonValue>, DbError> {
    let mut sql = format!(
        "SELECT to_jsonb(t) AS data FROM {} t WHERE 1=1",
        table.name()
    );
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(username.as_deref()) {
        sql.push_str(" AND t.username LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(start) = start_time {
        sql.push_str(" AND t.created_at >= ?");
        params.push(TypeMapper::to_value(&start));
    }
    if let Some(end) = end_time {
        sql.push_str(" AND t.created_at < ?");
        params.push(TypeMapper::to_value(&end));
    }
    sql.push_str(" ORDER BY t.created_at DESC");

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
        .iter()
        .map(|row| match row.get("data") {
            Some(value) => JsonValue::from_value(value),
            None => Err(DbError::ColumnNotFound("data".to_string())),
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}
