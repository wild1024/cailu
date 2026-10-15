//! 定时任务的强类型模型与数据访问。
//!
//! 对应迁移 `V0012__create_sys_job` 与 `V0013__create_sys_job_log`。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（分页、名称唯一性校验、启停写库）集中在**本文件**；仅调用
//! 框架快捷方法（`find_or_none` / `insert` / `update` / `delete` 等）的简单读写由
//! `handler.rs` 直接执行。

use alun::prelude::*;

use crate::shared::constants::EnableStatus;
use crate::utils;

/// 定时任务执行状态。
///
/// `#[int]` 使枚举以整数判别值落库，写 `Value::I32`、读回时兼容任意整数宽度。
///
/// 变体名沿用项目约定的 snake_case。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeMapper)]
#[int]
pub enum JobLogStatus {
    /// 失败（落库值 `0`）。
    failed = 0,
    /// 成功（落库值 `1`）。
    success = 1,
}

impl JobLogStatus {
    /// 转换为接口层使用的整数取值。
    ///
    /// # 返回值
    ///
    /// 成功返回 `1`，失败返回 `0`。
    pub fn as_i16(self) -> i16 {
        match self {
            JobLogStatus::success => 1,
            JobLogStatus::failed => 0,
        }
    }
}

/// `sys_job` 表对应的强类型模型（定时任务定义）。
///
/// # 状态语义
///
/// `status` 复用 [`EnableStatus`]：`enabled`（1）表示调度启用，
/// `disabled`（0）在该表中表示**暂停**。
///
/// # 唯一性与软删
///
/// `job_name` 的部分唯一索引带 `WHERE deleted_at IS NULL`。
#[derive(Debug, Clone, Model)]
#[table("sys_job")]
pub struct SysJobModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 任务名称，软删表上的部分唯一列。
    pub job_name: String,
    /// 任务分组（默认 `DEFAULT`）。
    pub job_group: String,
    /// cron 表达式。
    pub cron_expr: String,
    /// 处理器标识（任务处理器注册表的 key）。
    pub invoke_target: String,
    /// 任务参数（可空 JSONB）。
    #[json(nullable)]
    pub params: Option<JsonValue>,
    /// 任务状态（1 启用 / 0 暂停）。
    pub status: EnableStatus,
    /// 上次执行时间（从未执行为 `NULL`）。
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 下次执行时间（未排期为 `NULL`）。
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
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

/// `sys_job_log` 表对应的强类型模型（定时任务执行日志）。
///
/// # 追加写表
///
/// 表上仅有 `created_at`，无 `updated_at`，且无 `deleted_at`，因此派生标注
/// `#[model(no_timestamps, hard_delete)]`：不自动挂载 `TimestampHook`
/// （`created_at` 由写入方赋值），`delete` 走物理删除。
///
/// # 快照字段
///
/// `job_name` / `invoke_target` / `params` 为执行当时的冗余快照：即使任务定义
/// 后续被修改或删除，历史执行记录仍可独立还原。
#[derive(Debug, Clone, Model)]
#[table("sys_job_log")]
#[model(no_timestamps, hard_delete)]
pub struct SysJobLogModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 任务 ID（弱引用 `sys_job`，不建外键，保留历史记录）。
    pub job_id: uuid::Uuid,
    /// 任务名称（冗余快照）。
    pub job_name: String,
    /// 处理器标识（冗余快照）。
    pub invoke_target: String,
    /// 当次参数快照（可空 JSONB）。
    #[json(nullable)]
    pub params: Option<JsonValue>,
    /// 执行状态（1 成功 / 0 失败）。
    pub status: JobLogStatus,
    /// 结果摘要。
    pub message: Option<String>,
    /// 异常堆栈（失败时）。
    pub error: Option<String>,
    /// 开始时间。
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 结束时间（未结束为 `NULL`）。
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 耗时（毫秒）。
    pub duration_ms: Option<i32>,
    /// 创建时间（关闭时间戳 Hook，由写入方赋值）。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ==================== 查询条件与访问函数 ====================

/// 定时任务分页查询条件。
///
/// `job_name` 保持原始关键词，由本层转换为 `LIKE` 模式；`job_group` 精确匹配前
/// 去除首尾空白，空白值视为未提供。
pub(crate) struct JobFilter {
    /// 任务名称（模糊匹配）。
    pub job_name: Option<String>,
    /// 任务分组（精确匹配）。
    pub job_group: Option<String>,
    /// 状态过滤（1 启用 / 0 暂停）。
    pub status: Option<i16>,
}

/// 任务执行日志分页查询条件。
pub(crate) struct JobLogFilter {
    /// 任务 ID（精确匹配）。
    pub job_id: Option<uuid::Uuid>,
    /// 任务名称（模糊匹配）。
    pub job_name: Option<String>,
    /// 执行状态过滤（1 成功 / 0 失败）。
    pub status: Option<i16>,
}

/// 按任务名称查询未软删的任务（用于名称唯一性校验）。
///
/// # 参数
///
/// - `job_name`：任务名称。
///
/// # 返回值
///
/// 命中返回任务模型，未命中返回 `None`。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn find_job_by_name(
    job_name: &str,
) -> std::result::Result<Option<SysJobModel>, DbError> {
    SysJobModel::find_first_by(db(), "job_name = ? AND deleted_at IS NULL", (job_name,)).await
}

/// 更新定时任务状态（启用 / 暂停），并记录操作人与更新时间。
///
/// # 参数
///
/// - `id`：任务 ID。
/// - `status`：目标状态（`enabled` 启用 / `disabled` 暂停）。
/// - `operator`：当前操作人 ID（可为 `None`）。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn update_job_status(
    id: &uuid::Uuid,
    status: EnableStatus,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<(), DbError> {
    let now = chrono::Utc::now();
    db().execute_with_params(
        "UPDATE sys_job SET status = ?, update_by = ?, updated_at = ? WHERE id = ?",
        vec![
            TypeMapper::to_value(&status.as_i16()),
            Value::from(operator.as_ref()),
            TypeMapper::to_value(&now),
            Value::from(id),
        ],
    )
    .await
    .map(|_| ())
}

/// 分页查询定时任务。
///
/// # 参数
///
/// - `filter`：过滤条件（名称模糊 / 分组 / 状态）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的任务分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_jobs(
    filter: &JobFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysJobModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_job WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.job_name.as_deref()) {
        sql.push_str(" AND job_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(group) = filter
        .job_group
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sql.push_str(" AND job_group = ?");
        params.push(Value::from(group));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
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
        .map(SysJobModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 分页查询任务执行日志。
///
/// # 参数
///
/// - `filter`：过滤条件（任务 ID / 名称模糊 / 执行状态）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的日志分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_job_logs(
    filter: &JobLogFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysJobLogModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_job_log WHERE 1 = 1");
    let mut params: Vec<Value> = Vec::new();
    if let Some(job_id) = &filter.job_id {
        sql.push_str(" AND job_id = ?");
        params.push(Value::from(job_id));
    }
    if let Some(pattern) = utils::like_pattern(filter.job_name.as_deref()) {
        sql.push_str(" AND job_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
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
        .map(SysJobLogModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}
