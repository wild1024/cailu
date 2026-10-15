//! 定时任务与后台任务。
//!
//! 与业务模块分离，避免业务模块反向依赖调度细节。任务体可以调用任意业务模块的
//! `model.rs` / `handler.rs` 中已公开的函数。
//!
//! 调度采用框架内置的 [`alun::prelude::alun_plugin::scheduler::SchedulerPlugin`]（cron 表达式驱动，
//! 支持 5 段与 6 段写法）。框架调度器在 `start()` 时对注册表做**一次性快照**，
//! 启动后再 `register()` 不会生效；因此运行期增删任务必须走
//! `stop()` → 增删注册 → `start()`，该流程统一由 [`Scheduler::reload`] 完成。
//!
//! # 任务来源
//!
//! - **系统内置任务**：编译期固定在 [`Scheduler::reload`] 中注册；
//! - **动态任务**：从 `sys_job`（`status = 1`）读取后在 [`Scheduler::reload`] 中装载。
//!
//! | 任务 | cron | 处理器（invoke_target） | 交付阶段 |
//! |------|------|------------------------|----------|
//! | 操作日志清理 | `0 3 * * *` | `clean_operation_log` | P4 |
//! | 审计日志清理 | `10 3 * * *` | `clean_audit_log` | P4 |
//! | 认证日志清理 | `20 3 * * *` | `clean_auth_log` | P4 |
//! | 安全日志清理 | `30 3 * * *` | `clean_security_log` | P4 |
//! | 密码变更日志清理 | `40 3 * * *` | `clean_password_log` | P4 |
//!
//! 五张日志表错峰清理（均为每小时 3 点的不同分钟），避免同时发起批量删除抬高
//! 数据库瞬时压力。
//!
//! 在线用户会话存于缓存并按令牌有效期自动过期，因此**不实现**「在线用户清理」任务。
//!
//! [`scheduler`] 提供全局单例，由装配代码加入插件列表（`App::plugin`）并随应用
//! 生命周期启停；运行期由定时任务管理接口触发 [`Scheduler::reload`] 重新装载。

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use alun::prelude::alun_plugin::scheduler::{CronSchedule, SchedulerPlugin};
use alun::prelude::*;
use async_trait::async_trait;

use crate::shared::{constants, error_code};
use crate::support::job::model::{JobLogStatus, SysJobLogModel, SysJobModel};
use crate::support::log::model::{self as log_model, JsonLogTable};
use crate::utils;

/// 系统内置的日志清理任务：`(处理器标识 / 调度名, cron 表达式)`。
///
/// 标识同时用作调度名与 `sys_job.invoke_target`，因此三者保持一致，便于前端
/// 选择调用目标。
const LOG_CLEAN_TASKS: [(&str, &str); 5] = [
    ("clean_operation_log", "0 3 * * *"),
    ("clean_audit_log", "10 3 * * *"),
    ("clean_auth_log", "20 3 * * *"),
    ("clean_security_log", "30 3 * * *"),
    ("clean_password_log", "40 3 * * *"),
];

/// 把框架错误转换为统一的 500 内部错误（对外隐藏细节，细节写入日志）。
///
/// # 参数
///
/// - `e`：框架错误。
/// - `ctx`：业务上下文描述，用于日志定位。
///
/// # 返回值
///
/// 统一响应体为 `code = INTERNAL_ERROR`、消息为「服务器内部错误」的 [`ApiError`]。
fn scheduler_error(e: Error, ctx: &str) -> ApiError {
    tracing::error!(context = ctx, error = %e, "调度器操作失败");
    ApiError::new(500, error_code::INTERNAL_ERROR, "服务器内部错误")
}

/// 全局调度器：在框架 [`SchedulerPlugin`] 之上记录已注册的任务名，以便
/// [`Scheduler::reload`] 时先清理旧注册再重新装载。
///
/// 轻量克隆：内部字段均为 `Arc`，克隆后与本体共享同一调度器实例。
#[derive(Clone)]
pub struct Scheduler {
    /// 框架调度插件（`Arc` 包裹以便随本结构体一起轻量克隆）。
    inner: Arc<SchedulerPlugin>,
    /// 已注册到 `inner` 的任务名集合（含系统内置任务）。
    names: Arc<Mutex<HashSet<String>>>,
}

impl Scheduler {
    /// 创建空调度器（不注册任何任务）。
    ///
    /// 系统内置任务在 [`Scheduler::reload`] 中注册，这里仅创建空插件。
    ///
    /// # 返回值
    ///
    /// 未装载任何任务的调度器实例。
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SchedulerPlugin::new()),
            names: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// 重新装载全部任务。
    ///
    /// 执行顺序：`stop()` → 清理旧注册名 → 注册系统内置任务 → 从 `sys_job`
    /// 读取启用任务并注册 → `start()`。`sys_job` 查询失败时只记错误日志并保留
    /// 系统内置任务，不使装载失败。
    ///
    /// # 返回值
    ///
    /// 装载并启动成功返回 `Ok(())`。
    ///
    /// # 异常情况
    ///
    /// 调度器 `stop()` / `start()` 失败返回 500。
    pub async fn reload(&self) -> std::result::Result<(), ApiError> {
        // 1. 停止当前调度循环（框架 start() 对注册表做一次性快照，必须重载）
        self.inner
            .stop()
            .await
            .map_err(|e| scheduler_error(e, "停止调度器"))?;

        // 2. 清理上一次注册的任务名并逐个移除
        self.clear_registered();

        // 3. 注册系统内置任务
        self.register_builtin();

        // 4. 从数据库装载启用中的动态任务；查询失败不阻断启动
        match self.load_db_jobs().await {
            Ok(jobs) => {
                for job in jobs {
                    self.register_job(job);
                }
            }
            Err(e) => {
                tracing::error!(error = %e.msg, "装载定时任务失败，仅注册系统内置任务");
            }
        }

        // 5. 启动调度
        self.inner
            .start()
            .await
            .map_err(|e| scheduler_error(e, "启动调度器"))?;
        Ok(())
    }

    /// 立即执行一次指定任务，并记录执行日志、回写任务运行时间。
    ///
    /// # 参数
    ///
    /// - `job`：任务定义（含处理器标识、参数与 cron 表达式）。
    ///
    /// # 返回值
    ///
    /// 执行成功返回处理器的结果摘要文本。
    ///
    /// # 异常情况
    ///
    /// 处理器未注册或执行失败时返回对应 [`ApiError`]；日志写入与时间回写失败
    /// 只记错误日志，不影响返回值。
    pub async fn run_job(&self, job: &SysJobModel) -> std::result::Result<String, ApiError> {
        let started_at = chrono::Utc::now();
        let started = Instant::now();

        let outcome = run_handler(&job.invoke_target, job.params.clone()).await;

        let finished_at = chrono::Utc::now();
        let duration_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;

        // 无论成败都落一条执行日志
        self.write_log(job, &outcome, started_at, finished_at, duration_ms)
            .await;

        // 回写任务运行时间
        self.touch_job(job, finished_at).await;

        outcome
    }

    /// 触发框架调度器按已注册名字执行一次。
    ///
    /// # 参数
    ///
    /// - `name`：注册到框架调度器的任务名。
    ///
    /// # 返回值
    ///
    /// 找到对应注册项并成功启动返回 `true`，否则 `false`。
    pub fn trigger_registered(&self, name: &str) -> bool {
        self.inner.trigger(name).is_some()
    }

    /// 清空已注册任务名集合并逐个从框架调度器移除。
    fn clear_registered(&self) {
        let mut names = self
            .names
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        for name in names.drain() {
            self.inner.remove(&name);
        }
    }

    /// 记录一个已注册的任务名。
    ///
    /// # 参数
    ///
    /// - `name`：注册到框架调度器的任务名。
    fn remember(&self, name: &str) {
        self.names
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(name.to_string());
    }

    /// 注册系统内置任务。
    ///
    /// 内置任务无 `sys_job` 记录，执行结果只写运行日志（不写 `sys_job_log`）。
    /// 全部任务来自 [`LOG_CLEAN_TASKS`]，逐一注册。
    fn register_builtin(&self) {
        for (name, cron) in LOG_CLEAN_TASKS {
            let runner = move || tokio::spawn(run_builtin(name));
            self.inner
                .register(name, cron, "系统任务：日志清理", runner);
            self.remember(name);
        }
    }

    /// 注册单个动态任务（来自 `sys_job`）。
    ///
    /// 调度名取 `job_name`，cron 取 `cron_expr`，描述形如
    /// `系统任务：{job_name}`；runner 内克隆任务快照并在独立异步任务中调用
    /// [`Scheduler::run_job`]，失败只记日志。
    ///
    /// # 参数
    ///
    /// - `job`：任务定义。
    fn register_job(&self, job: SysJobModel) {
        let name = job.job_name.clone();
        let cron = job.cron_expr.clone();
        let desc = format!("系统任务：{}", job.job_name);
        let this = self.clone();
        let runner = move || {
            let this = this.clone();
            let job = job.clone();
            tokio::spawn(async move {
                if let Err(e) = this.run_job(&job).await {
                    tracing::error!(job = %job.job_name, error = %e.msg, "定时任务执行失败");
                }
            })
        };
        self.inner.register(&name, &cron, &desc, runner);
        self.remember(&name);
    }

    /// 从 `sys_job` 读取启用中的任务定义。
    ///
    /// # 返回值
    ///
    /// `status = 1` 且未软删的任务模型列表。
    ///
    /// # 异常情况
    ///
    /// 数据库异常返回 500。
    async fn load_db_jobs(&self) -> std::result::Result<Vec<SysJobModel>, ApiError> {
        let rows = db()
            .find_with_params(
                "SELECT * FROM sys_job WHERE deleted_at IS NULL AND status = ? \
                 ORDER BY created_at ASC",
                vec![TypeMapper::to_value(&constants::STATUS_ENABLED)],
            )
            .await
            .map_err(|e| utils::db_error(e, "查询启用定时任务"))?;
        rows.into_iter()
            .map(SysJobModel::from_row)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| utils::db_error(e, "映射定时任务模型"))
    }

    /// 写入一条任务执行日志（失败只记错误日志，不影响执行结果）。
    ///
    /// # 参数
    ///
    /// - `job`：任务定义（用于快照字段）。
    /// - `outcome`：处理器执行结果；成功取其摘要文本，失败取其错误信息，
    ///   并据此决定日志状态。
    /// - `started_at`：开始时间。
    /// - `finished_at`：结束时间。
    /// - `duration_ms`：耗时（毫秒）。
    async fn write_log(
        &self,
        job: &SysJobModel,
        outcome: &std::result::Result<String, ApiError>,
        started_at: chrono::DateTime<chrono::Utc>,
        finished_at: chrono::DateTime<chrono::Utc>,
        duration_ms: i32,
    ) {
        let (status, message, error) = match outcome {
            Ok(msg) => (JobLogStatus::success, Some(msg.clone()), None),
            Err(e) => (JobLogStatus::failed, None, Some(e.msg.clone())),
        };
        let mut log = SysJobLogModel {
            id: uuid::Uuid::now_v7(),
            job_id: job.id,
            job_name: job.job_name.clone(),
            invoke_target: job.invoke_target.clone(),
            params: job.params.clone(),
            status,
            message,
            error,
            started_at,
            finished_at: Some(finished_at),
            duration_ms: Some(duration_ms),
            // 追加写表关闭时间戳 Hook，created_at 必须显式赋值
            created_at: finished_at,
        };
        if let Err(e) = log.insert(db()).await {
            tracing::error!(job = %job.job_name, error = %e, "写入定时任务执行日志失败");
        }
    }

    /// 回写任务的 `last_run_at` / `next_run_at` / `updated_at`。
    ///
    /// `next_run_at` 由 cron 表达式推算，表达式不可解析或永无后续触发时刻时置为
    /// `NULL`。回写失败只记错误日志。
    ///
    /// # 参数
    ///
    /// - `job`：任务定义。
    /// - `finished_at`：本次执行结束时间，作为 `last_run_at` / `updated_at`。
    async fn touch_job(&self, job: &SysJobModel, finished_at: chrono::DateTime<chrono::Utc>) {
        let next_run_at = CronSchedule::parse(&job.cron_expr)
            .ok()
            .and_then(|schedule| schedule.next_after(chrono::Utc::now()));
        let next_value = match next_run_at {
            Some(value) => TypeMapper::to_value(&value),
            None => Value::Null,
        };
        let result = db()
            .execute_with_params(
                "UPDATE sys_job SET last_run_at = ?, next_run_at = ?, updated_at = ? WHERE id = ?",
                vec![
                    TypeMapper::to_value(&finished_at),
                    next_value,
                    TypeMapper::to_value(&finished_at),
                    Value::from(&job.id),
                ],
            )
            .await;
        if let Err(e) = result {
            tracing::error!(job = %job.job_name, error = %e, "回写定时任务运行时间失败");
        }
    }
}

impl Default for Scheduler {
    /// 等价于 [`Scheduler::new`]。
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for Scheduler {
    /// 插件名称（用于插件管理与日志）。
    fn name(&self) -> &str {
        "x-scheduler"
    }

    /// 启动调度：转发给框架 [`SchedulerPlugin`]。
    ///
    /// # 返回值
    ///
    /// 启动成功返回 `Ok(())`。
    ///
    /// # 异常情况
    ///
    /// 注册项中存在非法 cron 表达式等由框架返回错误。
    async fn start(&self) -> std::result::Result<(), Error> {
        self.inner.start().await
    }

    /// 关闭调度：转发给框架 [`SchedulerPlugin`]。
    ///
    /// # 返回值
    ///
    /// 关闭成功返回 `Ok(())`。
    ///
    /// # 异常情况
    ///
    /// 框架内部错误时返回错误。
    async fn stop(&self) -> std::result::Result<(), Error> {
        self.inner.stop().await
    }
}

/// 全局调度器单例。
static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();

/// 获取全局调度器单例（首次调用时惰性初始化）。
///
/// # 返回值
///
/// 全局调度器的轻量克隆（内部共享同一 `Arc<SchedulerPlugin>`）。
pub fn scheduler() -> Scheduler {
    SCHEDULER.get_or_init(Scheduler::new).clone()
}

/// 执行系统内置处理器并记录结果日志。
///
/// # 参数
///
/// - `invoke_target`：内置处理器标识（同时用作调度名）。
async fn run_builtin(invoke_target: &'static str) {
    match run_handler(invoke_target, None).await {
        Ok(message) => {
            tracing::info!(invoke_target = invoke_target, result = %message, "内置定时任务执行完成");
        }
        Err(e) => {
            tracing::error!(invoke_target = invoke_target, error = %e.msg, "内置定时任务执行失败");
        }
    }
}

/// 判断处理器标识是否已注册。
///
/// 内置处理器标识集中登记在 [`LOG_CLEAN_TASKS`]，供定时任务管理接口在新建 / 更新时
/// 校验 `invoke_target`，避免落库一个永远无法执行的调用目标。
///
/// # 参数
///
/// - `target`：处理器标识（调用目标）。
///
/// # 返回值
///
/// 已注册返回 `true`，否则 `false`。
pub fn is_registered_target(target: &str) -> bool {
    LOG_CLEAN_TASKS.iter().any(|(name, _)| *name == target)
}

/// 执行指定处理器。
///
/// 内置处理器为五张日志表的清理任务（见 [`LOG_CLEAN_TASKS`]）：各自删除本表中
/// 超过 [`constants::OPERATION_LOG_RETENTION_DAYS`] 天的记录，返回清理条数消息。
///
/// 内置处理器暂不使用 `params`，该参数保留给后续扩展。
///
/// # 参数
///
/// - `invoke_target`：处理器标识（调用目标）。
/// - `params`：任务参数（可为 `None`）。
///
/// # 返回值
///
/// 处理器执行成功返回结果摘要文本。
///
/// # 异常情况
///
/// - 422：处理器标识未注册；
/// - 500：处理器内部数据库操作失败。
pub async fn run_handler(
    invoke_target: &str,
    params: Option<JsonValue>,
) -> std::result::Result<String, ApiError> {
    tracing::debug!(invoke_target = invoke_target, params = ?params, "执行任务处理器");
    match invoke_target {
        "clean_operation_log" => {
            let affected = log_model::delete_operation_logs_before(log_cutoff())
                .await
                .map_err(|e| utils::db_error(e, "清理操作日志"))?;
            Ok(format!("已清理 {affected} 条操作日志"))
        }
        "clean_audit_log" => cleanup_json_logs(JsonLogTable::Audit, "审计日志").await,
        "clean_auth_log" => cleanup_json_logs(JsonLogTable::Auth, "认证日志").await,
        "clean_security_log" => cleanup_json_logs(JsonLogTable::Security, "安全日志").await,
        "clean_password_log" => {
            cleanup_json_logs(JsonLogTable::PasswordChange, "密码变更日志").await
        }
        other => Err(ApiError::new(
            422,
            error_code::JOB_TARGET_UNREGISTERED,
            format!("未注册的任务处理器: {other}"),
        )),
    }
}

/// 计算日志清理的时间阈值（当前时间减去统一保留天数）。
///
/// # 返回值
///
/// 早于该时刻的日志将被清理。
fn log_cutoff() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() - chrono::Duration::days(constants::OPERATION_LOG_RETENTION_DAYS)
}

/// 清理一张 JSON 日志表并返回结果摘要。
///
/// # 参数
///
/// - `table`：目标日志表。
/// - `label`：中文表名，用于拼接结果消息。
///
/// # 返回值
///
/// 形如「已清理 N 条审计日志」的结果摘要。
///
/// # 异常情况
///
/// 数据库异常返回 500。
async fn cleanup_json_logs(
    table: JsonLogTable,
    label: &str,
) -> std::result::Result<String, ApiError> {
    let affected = log_model::delete_json_logs_before(table, log_cutoff())
        .await
        .map_err(|e| utils::db_error(e, "清理日志"))?;
    Ok(format!("已清理 {affected} 条{label}"))
}
