//! 定时任务与执行日志的请求 / 响应 DTO。
//!
//! 字段名与前端契约（`frontend/src/api/support/job-api.ts`）严格对齐，避免联调时出现命名偏差。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::support::job::model::{SysJobLogModel, SysJobModel};

/// 定时任务响应。
///
/// `params` 为 JSON 文本（`None` 序列化为空串），`remark` 同理（`None` 为空串），
/// 与前端字符串字段保持一致。
#[derive(Debug, Clone, Serialize)]
pub struct JobRes {
    /// 任务 ID（UUID 字符串）。
    pub id: String,
    /// 任务名称。
    pub job_name: String,
    /// 任务分组。
    pub job_group: String,
    /// cron 表达式。
    pub cron_expr: String,
    /// 处理器标识（调用目标）。
    pub invoke_target: String,
    /// 调用参数（JSON 文本，无参数为空串）。
    pub params: String,
    /// 状态（1 启用 / 0 暂停）。
    pub status: i16,
    /// 上次执行时间（从未执行为 `None`）。
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 下次执行时间（未排期为 `None`）。
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 备注（无备注为空串）。
    pub remark: String,
    /// 创建时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&SysJobModel> for JobRes {
    /// 由模型构造响应。
    ///
    /// # 参数
    ///
    /// - `m`：定时任务模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的任务响应对象。
    fn from(m: &SysJobModel) -> Self {
        Self {
            id: m.id.to_string(),
            job_name: m.job_name.clone(),
            job_group: m.job_group.clone(),
            cron_expr: m.cron_expr.clone(),
            invoke_target: m.invoke_target.clone(),
            params: m.params.as_ref().map(|v| v.to_string()).unwrap_or_default(),
            status: m.status.as_i16(),
            last_run_at: m.last_run_at,
            next_run_at: m.next_run_at,
            remark: m.remark.clone().unwrap_or_default(),
            created_at: m.created_at,
        }
    }
}

/// 定时任务执行日志响应。
#[derive(Debug, Clone, Serialize)]
pub struct JobLogRes {
    /// 日志 ID（UUID 字符串）。
    pub id: String,
    /// 任务 ID（UUID 字符串）。
    pub job_id: String,
    /// 任务名称（快照）。
    pub job_name: String,
    /// 处理器标识（快照）。
    pub invoke_target: String,
    /// 执行状态（1 成功 / 0 失败）。
    pub status: i16,
    /// 结果摘要（无为空串）。
    pub message: String,
    /// 异常信息（无为空串）。
    pub error: String,
    /// 开始时间。
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 结束时间（未结束为 `None`）。
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 耗时（毫秒，缺省为 0）。
    pub duration_ms: i32,
    /// 记录时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&SysJobLogModel> for JobLogRes {
    /// 由模型构造响应。
    ///
    /// # 参数
    ///
    /// - `m`：执行日志模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的日志响应对象。
    fn from(m: &SysJobLogModel) -> Self {
        Self {
            id: m.id.to_string(),
            job_id: m.job_id.to_string(),
            job_name: m.job_name.clone(),
            invoke_target: m.invoke_target.clone(),
            status: m.status.as_i16(),
            message: m.message.clone().unwrap_or_default(),
            error: m.error.clone().unwrap_or_default(),
            started_at: m.started_at,
            finished_at: m.finished_at,
            duration_ms: m.duration_ms.unwrap_or(0),
            created_at: m.created_at,
        }
    }
}

/// 定时任务分页查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct JobPageQuery {
    /// 页码（缺省 1）。
    pub page: Option<u64>,
    /// 每页条数（缺省取框架默认值）。
    pub page_size: Option<u64>,
    /// 任务名称模糊关键词。
    pub job_name: Option<String>,
    /// 任务分组精确过滤。
    pub job_group: Option<String>,
    /// 状态过滤（1 启用 / 0 暂停）。
    pub status: Option<i16>,
}

/// 执行日志分页查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct JobLogPageQuery {
    /// 页码（缺省 1）。
    pub page: Option<u64>,
    /// 每页条数（缺省取框架默认值）。
    pub page_size: Option<u64>,
    /// 任务 ID 精确过滤。
    pub job_id: Option<String>,
    /// 任务名称模糊关键词。
    pub job_name: Option<String>,
    /// 执行状态过滤（1 成功 / 0 失败）。
    pub status: Option<i16>,
}

/// 定时任务新建 / 更新请求。
#[derive(Debug, Deserialize, Validate)]
pub struct JobSaveReq {
    /// 任务名称（软删表上的唯一列）。
    #[validate(length(min = 1, max = 64, message = "任务名称长度需为 1-64 个字符"))]
    pub job_name: String,
    /// 任务分组（缺省 `DEFAULT`）。
    #[validate(length(max = 64, message = "任务分组长度不能超过 64 个字符"))]
    pub job_group: Option<String>,
    /// cron 表达式。
    #[validate(length(min = 1, max = 64, message = "cron 表达式长度需为 1-64 个字符"))]
    pub cron_expr: String,
    /// 处理器标识（调用目标）。
    #[validate(length(min = 1, max = 255, message = "调用目标长度需为 1-255 个字符"))]
    pub invoke_target: String,
    /// 调用参数（JSON 文本或普通字符串，空串视为无参数）。
    pub params: Option<String>,
    /// 状态（1 启用 / 0 暂停），缺省启用。
    pub status: Option<i16>,
    /// 备注。
    #[validate(length(max = 500, message = "备注长度不能超过 500 个字符"))]
    pub remark: Option<String>,
}

/// 定时任务启停请求。
#[derive(Debug, Deserialize, Validate)]
pub struct JobStatusReq {
    /// 目标状态（1 启用 / 0 暂停）。
    pub status: i16,
}
