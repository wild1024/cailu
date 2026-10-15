//! 在线用户模块的请求与响应 DTO。

use alun::prelude::*;
use serde::{Deserialize, Serialize};

use crate::support::online::model::OnlineSession;

/// 在线用户列表项。
#[derive(Debug, Clone, Serialize)]
pub struct OnlineRes {
    /// 访问令牌唯一标识（强制下线时回传作为定位键）。
    pub token_id: String,
    /// 用户 ID。
    pub user_id: String,
    /// 用户名。
    pub username: String,
    /// 客户端 IP。
    pub ip: String,
    /// 登录地点。
    pub location: String,
    /// 浏览器类型。
    pub browser: String,
    /// 操作系统类型。
    pub os_type: String,
    /// 登录时间。
    pub login_time: chrono::DateTime<chrono::Utc>,
    /// 令牌到期时间。
    pub expire_at: chrono::DateTime<chrono::Utc>,
}

impl From<&OnlineSession> for OnlineRes {
    /// 由在线会话构造列表项。
    ///
    /// # 参数
    ///
    /// - `session`：在线会话快照。
    ///
    /// # 返回值
    ///
    /// 可空字段以空串兜底的 [`OnlineRes`]。
    fn from(session: &OnlineSession) -> Self {
        Self {
            token_id: session.token_id.clone(),
            user_id: session.user_id.clone(),
            username: session.username.clone(),
            ip: session.ip.clone().unwrap_or_default(),
            location: session.location.clone().unwrap_or_default(),
            browser: session.browser.clone().unwrap_or_default(),
            os_type: session.os_type.clone().unwrap_or_default(),
            login_time: session.login_time,
            expire_at: chrono::DateTime::from_timestamp(session.expire_at, 0)
                .unwrap_or(session.login_time),
        }
    }
}

/// 在线用户查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OnlineQuery {
    /// 页码（从 1 开始）。
    pub page: Option<u64>,
    /// 每页条数。
    pub page_size: Option<u64>,
    /// 用户名（模糊匹配）。
    pub username: Option<String>,
}
