//! 操作日志处理器：分页查询、详情、按时间清理。
//!
//! `operation_logs` 为追加写表（物理删除、无软删列）。处理器承担**参数校验、
//! 业务编排与响应组装**，并直接调用框架快捷方法（`find_or_none` 等）；需要编写
//! SQL 的查询（分页、清理）集中在 [`crate::support::log::model`]。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;

use crate::shared::constants;
use crate::shared::error_code;
use crate::support::log::dto::{CleanQuery, CleanRes, LogDetailRes, LogPageQuery, LogRes};
use crate::support::log::model::{self, JsonLogTable, OperationLogModel};
use crate::utils;

// ==================== 接口 ====================

/// 分页查询操作日志。
///
/// # 参数
///
/// - `query`：分页与过滤条件（用户名 / 操作 / 路径模糊，状态码精确，时间区间）。
///
/// # 返回值
///
/// 成功返回分页日志概要列表（按记录时间倒序）。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/page",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/page")]
pub async fn page(Query(query): Query<LogPageQuery>) -> ResResult<PageData<Vec<LogRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    let filter = model::OperationLogFilter {
        username: query.username,
        operation: query.operation,
        request_path: query.request_path,
        status_code: query.status_code,
        start_time: parse_optional_time(query.start_time.as_deref(), "start_time")?,
        end_time: parse_optional_time(query.end_time.as_deref(), "end_time")?,
    };

    let result = model::page_operation_logs(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询操作日志"))?;
    let list = result.list.iter().map(LogRes::from).collect::<Vec<_>>();

    Ok(Res::page(list, result.total, result.page, result.size))
}

/// 操作日志详情。
///
/// # 参数
///
/// - `id`：日志 ID。
///
/// # 返回值
///
/// 成功返回日志详情（含 `detail`、`request_params` 与 `response_data`）。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：日志不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/{id}",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<LogDetailRes> {
    let id = utils::require_uuid(&id, "id")?;
    let log = OperationLogModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询操作日志详情"))?
        .ok_or_else(|| ApiError::new(404, error_code::LOG_NOT_FOUND, "操作日志不存在"))?;
    Ok(Res::ok(LogDetailRes::from(&log)))
}

/// 清理指定天数之前的操作日志（物理删除）。
///
/// # 参数
///
/// - `query`：清理参数，`days` 为保留天数（缺省
///   [`constants::OPERATION_LOG_RETENTION_DAYS`]，至少为 1）。
///
/// # 返回值
///
/// 成功返回实际删除行数。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/support/log/clean",
    method = "DELETE",
    permission = "support:log:delete"
)]
#[alun::delete("/support/log/clean")]
pub async fn clean(Query(query): Query<CleanQuery>) -> ResResult<CleanRes> {
    let days = query
        .days
        .unwrap_or(constants::OPERATION_LOG_RETENTION_DAYS)
        .max(1);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);

    let removed = model::delete_operation_logs_before(cutoff)
        .await
        .map_err(|e| utils::db_error(e, "清理操作日志"))?;

    tracing::info!(days, removed, "已清理过期操作日志");
    Ok(Res::ok(CleanRes { removed }))
}

/// 分页查询审计日志。
///
/// 过滤条件为用户名模糊与时间区间，返回整行 JSON。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/audit/page",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/audit/page")]
pub async fn page_audit(Query(query): Query<LogPageQuery>) -> ResResult<PageData<Vec<JsonValue>>> {
    json_log_page(JsonLogTable::Audit, query).await
}

/// 分页查询认证日志。
///
/// 过滤条件为用户名模糊与时间区间，返回整行 JSON。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/auth/page",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/auth/page")]
pub async fn page_auth(Query(query): Query<LogPageQuery>) -> ResResult<PageData<Vec<JsonValue>>> {
    json_log_page(JsonLogTable::Auth, query).await
}

/// 分页查询安全日志。
///
/// 过滤条件为用户名模糊与时间区间，返回整行 JSON。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/security/page",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/security/page")]
pub async fn page_security(
    Query(query): Query<LogPageQuery>,
) -> ResResult<PageData<Vec<JsonValue>>> {
    json_log_page(JsonLogTable::Security, query).await
}

/// 分页查询密码变更日志。
///
/// 过滤条件为用户名模糊与时间区间，返回整行 JSON。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/log/password/page",
    method = "GET",
    permission = "support:log:list"
)]
#[alun::get("/support/log/password/page")]
pub async fn page_password(
    Query(query): Query<LogPageQuery>,
) -> ResResult<PageData<Vec<JsonValue>>> {
    json_log_page(JsonLogTable::PasswordChange, query).await
}

// ==================== 公共辅助 ====================

/// 解析可选的时间查询参数（未提供或空白时返回 `None`）。
///
/// # 参数
///
/// - `raw`：原始时间字符串。
/// - `field`：字段名，用于拼接错误消息。
///
/// # 异常情况
///
/// 提供了值但格式非法时返回 400 参数错误。
fn parse_optional_time(
    raw: Option<&str>,
    field: &str,
) -> std::result::Result<Option<chrono::DateTime<chrono::Utc>>, ApiError> {
    match utils::trimmed(raw) {
        Some(value) => parse_time(value, field).map(Some),
        None => Ok(None),
    }
}

/// 解析查询时间参数为 UTC 时间。
///
/// 先按 RFC3339（如 `2026-09-27T10:00:00Z`）解析，失败再按
/// `%Y-%m-%d %H:%M:%S` 解析并按 UTC 解释。
///
/// # 参数
///
/// - `raw`：时间字符串。
/// - `field`：字段名，用于拼接错误消息。
///
/// # 返回值
///
/// 解析成功返回 UTC 时间。
///
/// # 异常情况
///
/// 两种格式都解析失败返回 400 参数错误。
fn parse_time(
    raw: &str,
    field: &str,
) -> std::result::Result<chrono::DateTime<chrono::Utc>, ApiError> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S") {
        return Ok(naive.and_utc());
    }
    Err(ApiError::new(
        400,
        error_code::PARAM_INVALID,
        format!("{field} 时间格式不正确"),
    ))
}

/// 四类日志表共用的分页查询入口。
///
/// # 参数
///
/// - `table`：目标日志表（编译期固定枚举）。
/// - `query`：分页与过滤条件。
///
/// # 返回值
///
/// 分页后的 JSON 行列表（按 `created_at` 倒序）。
///
/// # 异常情况
///
/// - 400：时间参数格式非法；
/// - 500：数据库异常。
async fn json_log_page(
    table: JsonLogTable,
    query: LogPageQuery,
) -> ResResult<PageData<Vec<JsonValue>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);
    let start_time = parse_optional_time(query.start_time.as_deref(), "start_time")?;
    let end_time = parse_optional_time(query.end_time.as_deref(), "end_time")?;

    let result =
        model::page_json_logs(table, query.username, start_time, end_time, page, page_size)
            .await
            .map_err(|e| utils::db_error(e, "分页查询日志"))?;

    Ok(Res::page(
        result.list,
        result.total,
        result.page,
        result.size,
    ))
}
