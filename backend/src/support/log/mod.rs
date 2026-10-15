//! 日志子模块：操作 / 审计 / 认证 / 安全 / 密码变更五类日志。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | 五张日志表的强类型模型 |
//! | [`bus`] | 非阻塞日志总线（请求路径只投递，后台批量落库） |
//! | [`plugin`] | 日志总线插件（随应用生命周期启停） |
//! | [`dto`] | 日志请求 / 响应 DTO |
//! | [`handler`] | 分页查询 / 详情 / 清理接口 |
//!
//! 对外路径前缀 `/api/support/log`。`operation_logs` 由操作日志中间件写入，
//! 其余四张表由本模块的写入函数（[`record_auth`] / [`record_audit`] /
//! [`record_security`] / [`record_password_change`]）在登录、改密等路径调用。
//!
//! # 只写不改删
//!
//! 五张日志表统一为追加写语义：只有 `created_at`（无 `updated_at`）、物理删除、
//! 不提供修改接口；清理按时间窗口批量删除。
//!
//! # 非阻塞写入
//!
//! 四个 `record_*` 函数均为**同步、无阻塞**：只做一次事件投递即返回，
//! 不触碰数据库、不解析 UA、不查询 IP 归属地。真正的落库由 [`bus`] 的后台
//! worker 批量完成，因此日志写入不会拖慢登录等主业务。
//!
//! # 写入容错
//!
//! 日志写入失败只记 `tracing::error!`，**绝不影响调用方**（登录失败/成功的结果
//! 不因日志落库失败而改变），与 [`crate::middleware::operation_log`] 的策略一致。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/api/support/log/page` | `support:log:list` | 操作日志分页 |
//! | GET | `/api/support/log/{id}` | `support:log:list` | 操作日志详情 |
//! | DELETE | `/api/support/log/clean` | `support:log:delete` | 清理早于保留期的操作日志 |
//! | GET | `/api/support/log/audit/page` | `support:log:list` | 审计日志分页 |
//! | GET | `/api/support/log/auth/page` | `support:log:list` | 认证日志分页 |
//! | GET | `/api/support/log/security/page` | `support:log:list` | 安全日志分页 |
//! | GET | `/api/support/log/password/page` | `support:log:list` | 密码变更日志分页 |

pub(crate) mod bus;
pub mod dto;
pub mod handler;
pub mod model;
pub(crate) mod plugin;

use alun::prelude::*;
use axum::http::HeaderMap;
use std::net::SocketAddr;

use crate::support::log::bus::AuthEvent;
use crate::support::log::model::{AuditLogModel, PasswordChangeLogModel, SecurityLogModel};
use crate::utils;

// ==================== 认证类型 / 状态的受控取值 ====================

/// 认证类型（`auth_logs.auth_type`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthType {
    /// 登录。
    Login,
    /// 登出。
    Logout,
    /// 刷新令牌。
    RefreshToken,
}

impl AuthType {
    /// 落库取值。
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AuthType::Login => "LOGIN",
            AuthType::Logout => "LOGOUT",
            AuthType::RefreshToken => "REFRESH_TOKEN",
        }
    }
}

/// 认证结果（`auth_logs.auth_status`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthStatus {
    /// 成功。
    Success,
    /// 失败。
    Fail,
}

impl AuthStatus {
    /// 落库取值。
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AuthStatus::Success => "SUCCESS",
            AuthStatus::Fail => "FAIL",
        }
    }
}

// ==================== 写入入参 ====================

/// 认证日志写入入参（请求头派生的字段由 [`record_auth`] 自动填充）。
pub(crate) struct AuthLogInput {
    /// 登录用户 ID（用户不存在时为 `None`）。
    pub user_id: Option<uuid::Uuid>,
    /// 登录用户名。
    pub username: Option<String>,
    /// 认证类型。
    pub auth_type: AuthType,
    /// 认证结果。
    pub auth_status: AuthStatus,
    /// 失败原因（成功时为 `None`）。
    pub error_message: Option<String>,
    /// 登录耗时（毫秒）。
    pub login_duration: Option<i32>,
}

/// 审计日志写入入参。
pub(crate) struct AuditLogInput {
    /// 操作用户 ID。
    pub user_id: Option<uuid::Uuid>,
    /// 操作用户名（冗余）。
    pub username: Option<String>,
    /// 操作类型（如 `USER_CREATE`）。
    pub operation: String,
    /// 操作对象类型（如 `User`）。
    pub target_type: Option<String>,
    /// 操作对象 ID。
    pub target_id: Option<String>,
    /// 变更前数据快照（新建时为 `None`）。
    pub before_data: Option<JsonValue>,
    /// 变更后数据快照（删除时为 `None`）。
    pub after_data: Option<JsonValue>,
    /// 补充详情。
    pub detail: Option<JsonValue>,
}

/// 安全日志写入入参。
pub(crate) struct SecurityLogInput {
    /// 关联用户 ID（可能为空）。
    pub user_id: Option<uuid::Uuid>,
    /// 用户名（尝试登录或操作的用户名）。
    pub username: Option<String>,
    /// 事件类型（如 `LOGIN_FAIL`）。
    pub event_type: String,
    /// 事件详情（脱敏）。
    pub detail: Option<JsonValue>,
}

/// 密码变更日志写入入参。
pub(crate) struct PasswordChangeLogInput {
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
    /// 变更状态：`SUCCESS` / `FAIL`。
    pub change_status: String,
    /// 失败原因。
    pub error_message: Option<String>,
}

// ==================== 写入函数 ====================

/// 投递认证日志事件（无阻塞）。
///
/// # 参数
///
/// - `headers`：请求头（派生 request_id / IP / User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP 用）。
/// - `input`：认证相关的业务字段。
///
/// # 说明
///
/// UA 解析与 IP 归属地查询在后台 worker 完成，本函数不做任何 I/O，
/// 因此**不会**影响调用方（登录等）的处理速度。
pub(crate) fn record_auth(headers: &HeaderMap, connect_info: &SocketAddr, input: AuthLogInput) {
    bus::enqueue_auth(AuthEvent {
        request_id: utils::request_id(headers),
        user_id: input.user_id,
        username: input.username,
        auth_type: input.auth_type.as_str(),
        auth_status: input.auth_status.as_str(),
        ip: utils::client_ip(headers, Some(connect_info)),
        user_agent: utils::user_agent(headers),
        error_message: input.error_message,
        login_duration: input.login_duration,
        created_at: chrono::Utc::now(),
    });
}

/// 投递审计日志事件（无阻塞）。
///
/// # 参数
///
/// - `headers`：请求头（派生 request_id / IP / User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP 用）。
/// - `input`：审计相关的业务字段。
pub(crate) fn record_audit(headers: &HeaderMap, connect_info: &SocketAddr, input: AuditLogInput) {
    bus::enqueue_audit(AuditLogModel {
        id: uuid::Uuid::now_v7(),
        request_id: utils::request_id(headers),
        user_id: input.user_id,
        username: input.username,
        operation: input.operation,
        target_type: input.target_type,
        target_id: input.target_id,
        before_data: input.before_data,
        after_data: input.after_data,
        detail: input.detail,
        ip: utils::client_ip(headers, Some(connect_info)),
        user_agent: utils::user_agent(headers),
        created_at: chrono::Utc::now(),
    });
}

/// 便捷构造审计日志入参（操作者信息取自令牌声明）。
///
/// 供各业务模块 handler 在新增 / 变更 / 删除后上报审计日志时复用，避免逐处
/// 手写操作者字段。
///
/// # 参数
///
/// - `claims`：当前用户令牌声明（派生 `user_id` / `username`）。
/// - `operation`：操作类型（如 `USER_CREATE`）。
/// - `target_type`：操作对象类型（如 `User`）。
/// - `target_id`：操作对象 ID。
/// - `before_data`：变更前数据快照（新建时为 `None`）。
/// - `after_data`：变更后数据快照（删除时为 `None`）。
///
/// # 返回值
///
/// 除 `detail` 固定为 `None` 外，其余字段均来自入参的 [`AuditLogInput`]。
pub(crate) fn audit_input(
    claims: &TokenClaims,
    operation: &str,
    target_type: &str,
    target_id: String,
    before_data: Option<JsonValue>,
    after_data: Option<JsonValue>,
) -> AuditLogInput {
    AuditLogInput {
        user_id: utils::current_user_uuid(claims),
        username: claims.username.clone(),
        operation: operation.to_string(),
        target_type: Some(target_type.to_string()),
        target_id: Some(target_id),
        before_data,
        after_data,
        detail: None,
    }
}

/// 投递安全日志事件（无阻塞）。
///
/// # 参数
///
/// - `headers`：请求头（派生 request_id / IP / User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP 用）。
/// - `input`：安全事件字段。
pub(crate) fn record_security(
    headers: &HeaderMap,
    connect_info: &SocketAddr,
    input: SecurityLogInput,
) {
    bus::enqueue_security(SecurityLogModel {
        id: uuid::Uuid::now_v7(),
        request_id: Some(utils::request_id(headers)),
        user_id: input.user_id,
        username: input.username,
        event_type: input.event_type,
        detail: input.detail,
        ip: utils::client_ip(headers, Some(connect_info)),
        user_agent: utils::user_agent(headers),
        created_at: chrono::Utc::now(),
    });
}

/// 投递密码变更日志事件（无阻塞）。
///
/// # 参数
///
/// - `headers`：请求头（派生 request_id / IP / User-Agent）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP 用）。
/// - `input`：密码变更字段。
pub(crate) fn record_password_change(
    headers: &HeaderMap,
    connect_info: &SocketAddr,
    input: PasswordChangeLogInput,
) {
    bus::enqueue_password_change(PasswordChangeLogModel {
        id: uuid::Uuid::now_v7(),
        request_id: utils::request_id(headers),
        user_id: input.user_id,
        username: input.username,
        change_type: input.change_type,
        operator_type: input.operator_type,
        operator_id: input.operator_id,
        ip: utils::client_ip(headers, Some(connect_info)),
        user_agent: utils::user_agent(headers),
        change_status: input.change_status,
        error_message: input.error_message,
        created_at: chrono::Utc::now(),
    });
}
