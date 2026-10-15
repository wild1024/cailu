//! 操作日志的请求与响应 DTO。
//!
//! 字段名与前端契约 `frontend/src/api/support/log-api.ts` 严格对齐：列表只下发概要字段，
//! 详情在概要基础上追加 `detail` / `request_params` / `response_data` 三个扩展字段。

use alun::prelude::*;
use serde::{Deserialize, Serialize};

use crate::support::log::model::OperationLogModel;

/// 操作日志概要响应（列表项）。
///
/// 可空列统一以空串 / `0` 兜底，保证前端拿到稳定的类型。
#[derive(Debug, Clone, Serialize)]
pub struct LogRes {
    /// 日志 ID（UUID 字符串）。
    pub id: String,
    /// 请求追踪 ID。
    pub request_id: String,
    /// 操作用户 ID。
    pub user_id: String,
    /// 操作用户名。
    pub username: String,
    /// 操作描述（如 `USER_CREATE`）。
    pub operation: String,
    /// 目标资源类型。
    pub target_type: String,
    /// 目标资源 ID。
    pub target_id: String,
    /// 客户端 IP。
    pub ip: String,
    /// User-Agent。
    pub user_agent: String,
    /// 处理耗时（毫秒）。
    pub duration_ms: i32,
    /// 请求路径。
    pub request_path: String,
    /// 请求方法。
    pub request_method: String,
    /// 响应状态码。
    pub status_code: i32,
    /// 记录时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&OperationLogModel> for LogRes {
    /// 由操作日志模型构造概要响应。
    ///
    /// # 参数
    ///
    /// - `m`：操作日志模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的 [`LogRes`]；可空字段分别以空串 / `0` 兜底。
    fn from(m: &OperationLogModel) -> Self {
        Self {
            id: m.id.to_string(),
            request_id: m.request_id.clone(),
            user_id: m.user_id.map(|v| v.to_string()).unwrap_or_default(),
            username: m.username.clone().unwrap_or_default(),
            operation: m.operation.clone(),
            target_type: m.target_type.clone().unwrap_or_default(),
            target_id: m.target_id.clone().unwrap_or_default(),
            ip: m.ip.map(|v| v.to_string()).unwrap_or_default(),
            user_agent: m.user_agent.clone().unwrap_or_default(),
            duration_ms: m.duration_ms.unwrap_or(0),
            request_path: m.request_path.clone().unwrap_or_default(),
            request_method: m.request_method.clone().unwrap_or_default(),
            status_code: m.status_code.unwrap_or(0),
            created_at: m.created_at,
        }
    }
}

/// 操作日志详情响应。
///
/// 在概要字段基础上追加 `detail`（脱敏后的详情 JSON 文本）、`request_params`
/// （脱敏后的请求体）与 `response_data`（脱敏后的响应体）。`log` 以
/// `#[serde(flatten)]` 内联展开，序列化结果与前端 `LogDetailRes extends LogRes`
/// 的结构一致。
#[derive(Debug, Clone, Serialize)]
pub struct LogDetailRes {
    /// 概要字段（内联展开到同一层级）。
    #[serde(flatten)]
    pub log: LogRes,
    /// 详情描述（`Option<JsonValue>` 序列化为紧凑 JSON 文本，缺省为空串）。
    pub detail: String,
    /// 请求参数（脱敏后的 JSON，缺省为 JSON `null`）。
    pub request_params: JsonValue,
    /// 响应数据（脱敏后的 JSON，缺省为 JSON `null`）。
    pub response_data: JsonValue,
}

impl From<&OperationLogModel> for LogDetailRes {
    /// 由操作日志模型构造详情响应。
    ///
    /// # 参数
    ///
    /// - `m`：操作日志模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的 [`LogDetailRes`]；`detail` 缺省为空串，`request_params` /
    /// `response_data` 缺省为 `null`。
    fn from(m: &OperationLogModel) -> Self {
        Self {
            log: LogRes::from(m),
            detail: m.detail.clone().map(|v| v.to_string()).unwrap_or_default(),
            request_params: m.request_params.clone().unwrap_or(JsonValue::Null),
            response_data: m.response_data.clone().unwrap_or(JsonValue::Null),
        }
    }
}

/// 操作日志分页查询参数。
///
/// `page` / `page_size` 缺省时由 [`crate::utils::normalize_page`] 规整；
/// `start_time` / `end_time` 为字符串，同时兼容 RFC3339 与
/// `%Y-%m-%d %H:%M:%S` 两种格式（后者按 UTC 解释）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogPageQuery {
    /// 页码（从 1 开始）。
    pub page: Option<u64>,
    /// 每页条数。
    pub page_size: Option<u64>,
    /// 操作用户名（模糊匹配）。
    pub username: Option<String>,
    /// 操作描述（模糊匹配）。
    pub operation: Option<String>,
    /// 请求路径（模糊匹配）。
    pub request_path: Option<String>,
    /// 响应状态码（精确匹配）。
    pub status_code: Option<i32>,
    /// 起始时间（含）。
    pub start_time: Option<String>,
    /// 结束时间（不含）。
    pub end_time: Option<String>,
}

/// 日志清理查询参数。
///
/// `days` 表示保留天数，缺省取 [`crate::shared::constants::OPERATION_LOG_RETENTION_DAYS`]；
/// 至少为 1，小于 1 时按 1 处理。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CleanQuery {
    /// 保留天数（清理该天数之前的日志）。
    pub days: Option<i64>,
}

/// 日志清理结果。
#[derive(Debug, Clone, Serialize)]
pub struct CleanRes {
    /// 实际删除的日志条数。
    pub removed: u64,
}
