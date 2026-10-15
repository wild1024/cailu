//! 非阻塞日志总线：把日志写入从请求路径剥离到后台批量落库。
//!
//! # 设计目标
//!
//! 「日志记录不得干扰主业务处理速度」是硬约束。因此业务侧只做一次
//! **无阻塞投递**——把事件放进无界通道即返回，不等待数据库、不解析 UA、
//! 不查询 IP 归属地；后台 worker 负责聚合事件、按批次调用
//! [`PersistExt::insert_many`] 批量落库。
//!
//! 如此请求路径上只剩「构造轻量模型 + 一次 channel send」，UA 解析（CPU）、
//! IP 归属地外呼（网络）与数据库往返全部移出请求线程。
//!
//! # 背压与丢弃
//!
//! 通道为无界队列：日志是「尽力而为」的旁路数据，不允许因日志堆积反压业务。
//! 内存风险由「worker 持续消费 + 到达阈值即 flush」化解。
//!
//! # 生命周期
//!
//! - [`init`]：由日志插件在 `start()` 调用，安装全局发送端并拉起 worker；
//! - [`flush`]：由日志插件在 `stop()` 调用，等待已入队事件全部落库。
//!
//! 未显式 [`init`] 时，首次投递会惰性初始化（仍要求处于 tokio 运行时内）。

use std::sync::OnceLock;
use std::time::Duration;

use alun::prelude::*;
use tokio::sync::{mpsc, oneshot};

use crate::support::log::model::{
    AuditLogModel, AuthLogModel, OperationLogModel, PasswordChangeLogModel, SecurityLogModel,
};

/// 批量落库阈值：任一类型待写行数达到该值即触发一次 flush。
const MAX_BATCH_ROWS: usize = 256;

/// 定时 flush 间隔：即使未达批量阈值，也会在此时长内落库一次。
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);

/// [`flush`] 等待 worker 回执的最长时长。
const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// 全局日志事件发送端（由 [`init`] 或首次投递时安装）。
static SENDER: OnceLock<mpsc::UnboundedSender<LogEvent>> = OnceLock::new();

/// 认证日志事件（原始数据：UA 解析与 IP 归属地查询留给后台 worker）。
pub(crate) struct AuthEvent {
    /// 请求追踪 ID。
    pub request_id: String,
    /// 用户 ID（用户不存在时为 `None`）。
    pub user_id: Option<uuid::Uuid>,
    /// 用户名。
    pub username: Option<String>,
    /// 认证类型落库值。
    pub auth_type: &'static str,
    /// 认证结果落库值。
    pub auth_status: &'static str,
    /// 客户端 IP。
    pub ip: Option<std::net::IpAddr>,
    /// 原始 User-Agent。
    pub user_agent: Option<String>,
    /// 失败原因（成功时为 `None`）。
    pub error_message: Option<String>,
    /// 登录耗时（毫秒）。
    pub login_duration: Option<i32>,
    /// 事件时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 日志事件。
enum LogEvent {
    /// 操作日志（模型已在业务侧构造，构造开销仅为字段拷贝）。
    Operation(OperationLogModel),
    /// 认证日志（原始数据，worker 侧解析 UA 与 IP 归属地）。
    Auth(AuthEvent),
    /// 审计日志。
    Audit(AuditLogModel),
    /// 安全日志。
    Security(SecurityLogModel),
    /// 密码变更日志。
    PasswordChange(PasswordChangeLogModel),
    /// 刷盘哨兵：worker 完成一次 flush 后回执，供 [`flush`] 等待。
    Flush(oneshot::Sender<()>),
}

/// 初始化日志总线（幂等）。
///
/// 安装全局发送端并拉起后台 worker。必须在 tokio 运行时内调用；不在运行时内
/// 时记错误日志并放弃初始化（投递会一并丢弃并告警）。
pub(crate) fn init() {
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        tracing::error!("日志总线初始化失败：当前不在 tokio 运行时内");
        return;
    };
    SENDER.get_or_init(|| {
        let (sender, receiver) = mpsc::unbounded_channel();
        handle.spawn(worker_loop(receiver));
        sender
    });
}

/// 把日志事件投递到总线（无阻塞）。
///
/// # 参数
///
/// - `event`：日志事件。
///
/// # 异常情况
///
/// 通道已关闭或不在异步上下文时丢弃事件并记告警——日志失败绝不波及业务。
fn enqueue(event: LogEvent) {
    if let Some(sender) = SENDER.get() {
        if sender.send(event).is_err() {
            tracing::warn!("日志总线已关闭，事件被丢弃");
        }
        return;
    }

    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        tracing::error!("日志总线在非异步上下文中投递，事件被丢弃");
        return;
    };
    let sender = SENDER.get_or_init(|| {
        let (sender, receiver) = mpsc::unbounded_channel();
        handle.spawn(worker_loop(receiver));
        sender
    });
    if sender.send(event).is_err() {
        tracing::warn!("日志总线已关闭，事件被丢弃");
    }
}

/// 投递一条操作日志事件。
///
/// # 参数
///
/// - `model`：已构造的操作日志模型。
pub(crate) fn enqueue_operation(model: OperationLogModel) {
    enqueue(LogEvent::Operation(model));
}

/// 投递一条审计日志事件。
///
/// # 参数
///
/// - `model`：已构造的审计日志模型。
pub(crate) fn enqueue_audit(model: AuditLogModel) {
    enqueue(LogEvent::Audit(model));
}

/// 投递一条安全日志事件。
///
/// # 参数
///
/// - `model`：已构造的安全日志模型。
pub(crate) fn enqueue_security(model: SecurityLogModel) {
    enqueue(LogEvent::Security(model));
}

/// 投递一条密码变更日志事件。
///
/// # 参数
///
/// - `model`：已构造的密码变更日志模型。
pub(crate) fn enqueue_password_change(model: PasswordChangeLogModel) {
    enqueue(LogEvent::PasswordChange(model));
}

/// 投递一条认证日志事件。
///
/// # 参数
///
/// - `event`：认证事件原始数据（UA 解析与 IP 归属地在 worker 侧完成）。
pub(crate) fn enqueue_auth(event: AuthEvent) {
    enqueue(LogEvent::Auth(event));
}

/// 等待已入队事件全部落库（进程退出前由日志插件调用）。
///
/// 通过向 worker 发送刷盘哨兵并等待回执实现：回执到达即代表哨兵之前的
/// 所有事件均已写库。
pub(crate) async fn flush() {
    let Some(sender) = SENDER.get() else {
        return;
    };
    let (ack_sender, ack_receiver) = oneshot::channel();
    if sender.send(LogEvent::Flush(ack_sender)).is_err() {
        tracing::warn!("日志总线已关闭，跳过刷盘");
        return;
    }
    if tokio::time::timeout(FLUSH_TIMEOUT, ack_receiver)
        .await
        .is_err()
    {
        tracing::warn!("等待日志刷盘超时，部分日志可能尚未落库");
    }
}

/// 后台 worker：聚合事件、批量落库。
///
/// 循环消费事件，任一类型批次到达 [`MAX_BATCH_ROWS`] 或到达 [`FLUSH_INTERVAL`]
/// 即落库；收到 [`LogEvent::Flush`] 时落库后回执；通道关闭时做最后一次落库后退出。
async fn worker_loop(mut receiver: mpsc::UnboundedReceiver<LogEvent>) {
    let mut batches = Batches::default();
    let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            incoming = receiver.recv() => match incoming {
                Some(LogEvent::Flush(ack)) => {
                    batches.flush().await;
                    let _ = ack.send(());
                }
                Some(event) => {
                    batches.push(event).await;
                    if batches.is_full() {
                        batches.flush().await;
                    }
                }
                None => {
                    batches.flush().await;
                    break;
                }
            },
            _ = ticker.tick() => {
                batches.flush().await;
            }
        }
    }
}

/// 五类日志的待写批次。
#[derive(Default)]
struct Batches {
    /// 操作日志批次。
    operations: Vec<OperationLogModel>,
    /// 认证日志批次。
    auths: Vec<AuthLogModel>,
    /// 审计日志批次。
    audits: Vec<AuditLogModel>,
    /// 安全日志批次。
    securities: Vec<SecurityLogModel>,
    /// 密码变更日志批次。
    passwords: Vec<PasswordChangeLogModel>,
}

impl Batches {
    /// 把事件并入对应批次；认证事件在此完成 UA 解析与归属地解析。
    ///
    /// # 参数
    ///
    /// - `event`：日志事件。
    async fn push(&mut self, event: LogEvent) {
        match event {
            LogEvent::Operation(model) => self.operations.push(model),
            LogEvent::Audit(model) => self.audits.push(model),
            LogEvent::Security(model) => self.securities.push(model),
            LogEvent::PasswordChange(model) => self.passwords.push(model),
            LogEvent::Auth(event) => self.auths.push(build_auth_model(event).await),
            // 哨兵事件已在上层处理，不会走到这里
            LogEvent::Flush(_) => {}
        }
    }

    /// 是否存在达到批量阈值的批次。
    ///
    /// # 返回值
    ///
    /// 任一类型批次达到 [`MAX_BATCH_ROWS`] 返回 `true`。
    fn is_full(&self) -> bool {
        self.operations.len() >= MAX_BATCH_ROWS
            || self.auths.len() >= MAX_BATCH_ROWS
            || self.audits.len() >= MAX_BATCH_ROWS
            || self.securities.len() >= MAX_BATCH_ROWS
            || self.passwords.len() >= MAX_BATCH_ROWS
    }

    /// 逐类型批量落库并清空批次。
    ///
    /// 单类型写入失败只记错误日志并丢弃该批次，不影响其它类型与业务主流程。
    async fn flush(&mut self) {
        let Some(database) = try_db() else {
            if self.has_pending() {
                tracing::error!("数据库未就绪，丢弃待写日志批次");
                self.clear();
            }
            return;
        };

        if !self.operations.is_empty() {
            let rows = std::mem::take(&mut self.operations);
            if let Err(e) = OperationLogModel::insert_many(&rows, database).await {
                tracing::error!(count = rows.len(), error = %e, "批量写入操作日志失败");
            }
        }
        if !self.auths.is_empty() {
            let rows = std::mem::take(&mut self.auths);
            if let Err(e) = AuthLogModel::insert_many(&rows, database).await {
                tracing::error!(count = rows.len(), error = %e, "批量写入认证日志失败");
            }
        }
        if !self.audits.is_empty() {
            let rows = std::mem::take(&mut self.audits);
            if let Err(e) = AuditLogModel::insert_many(&rows, database).await {
                tracing::error!(count = rows.len(), error = %e, "批量写入审计日志失败");
            }
        }
        if !self.securities.is_empty() {
            let rows = std::mem::take(&mut self.securities);
            if let Err(e) = SecurityLogModel::insert_many(&rows, database).await {
                tracing::error!(count = rows.len(), error = %e, "批量写入安全日志失败");
            }
        }
        if !self.passwords.is_empty() {
            let rows = std::mem::take(&mut self.passwords);
            if let Err(e) = PasswordChangeLogModel::insert_many(&rows, database).await {
                tracing::error!(count = rows.len(), error = %e, "批量写入密码变更日志失败");
            }
        }
    }

    /// 是否存在待写数据。
    ///
    /// # 返回值
    ///
    /// 任一类型批次非空返回 `true`。
    fn has_pending(&self) -> bool {
        !self.operations.is_empty()
            || !self.auths.is_empty()
            || !self.audits.is_empty()
            || !self.securities.is_empty()
            || !self.passwords.is_empty()
    }

    /// 清空全部批次。
    fn clear(&mut self) {
        self.operations.clear();
        self.auths.clear();
        self.audits.clear();
        self.securities.clear();
        self.passwords.clear();
    }
}

/// 由认证事件构造认证日志模型（UA 解析 + IP 归属地解析）。
///
/// # 参数
///
/// - `event`：认证事件原始数据。
///
/// # 返回值
///
/// 可直接落库的 [`AuthLogModel`]。
async fn build_auth_model(event: AuthEvent) -> AuthLogModel {
    let (device_type, browser_type, os_type) = match event.user_agent.as_deref() {
        Some(raw) => {
            let info = alun_utils::parse_user_agent(raw);
            (
                Some(info.device_type),
                Some(info.browser_type),
                Some(info.os_type),
            )
        }
        None => (None, None, None),
    };
    let location = match event.ip {
        Some(ip) => crate::support::ipgeo::resolve(ip).await,
        None => None,
    };

    AuthLogModel {
        id: uuid::Uuid::now_v7(),
        request_id: event.request_id,
        user_id: event.user_id,
        username: event.username,
        auth_type: event.auth_type.to_string(),
        auth_status: event.auth_status.to_string(),
        ip: event.ip,
        location,
        device_type,
        browser_type,
        os_type,
        user_agent: event.user_agent,
        error_message: event.error_message,
        login_duration: event.login_duration,
        created_at: event.created_at,
    }
}
