//! 在线用户会话（缓存承载）。
//!
//! 会话不落数据库：以「访问令牌的 `jti`」为键写入全局 `cache()`
//! （`online:token:{jti}`，TTL 对齐令牌剩余有效期），从而：
//!
//! - 令牌过期即自动从在线列表消失，无需清理任务；
//! - 登出 / 强制下线只需删除键并写入框架同格式的令牌黑名单。
//!
//! 数据访问与领域行为集中在本文件，处理器只做业务编排。

use alun::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::constants;

/// 在线用户会话快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineSession {
    /// 访问令牌唯一标识（`jti`），同时是缓存键后缀。
    pub token_id: String,
    /// 用户 ID（令牌 `sub`）。
    pub user_id: String,
    /// 用户名。
    pub username: String,
    /// 客户端 IP 文本。
    pub ip: Option<String>,
    /// 登录地点（归属地缓存命中时填充，可能为空）。
    pub location: Option<String>,
    /// 浏览器类型。
    pub browser: Option<String>,
    /// 操作系统类型。
    pub os_type: Option<String>,
    /// 登录时间。
    pub login_time: chrono::DateTime<chrono::Utc>,
    /// 令牌到期时间（Unix 时间戳，秒）。
    pub expire_at: i64,
}

/// 在线会话数据访问（缓存命名空间）。
pub struct OnlineSessionModel;

impl OnlineSessionModel {
    /// 登记一条在线会话。
    ///
    /// # 参数
    ///
    /// - `session`：会话快照。
    /// - `ttl_secs`：缓存有效期（秒），应取令牌剩余有效期。
    ///
    /// # 异常情况
    ///
    /// 缓存未初始化或写入失败时返回错误，由调用方决定是否中断主流程。
    pub async fn record_login(
        session: &OnlineSession,
        ttl_secs: u64,
    ) -> std::result::Result<(), Error> {
        let Some(store) = try_cache() else {
            return Err(Error::Msg("缓存未初始化，无法登记在线会话".to_string()));
        };
        let key = Self::key(&session.token_id);
        Cache::set_ex(store, &key, session, ttl_secs).await
    }

    /// 移除一条在线会话（登出 / 会话结束时调用）。
    ///
    /// 键不存在不视为错误。
    ///
    /// # 参数
    ///
    /// - `token_id`：访问令牌 `jti`。
    pub async fn record_logout(token_id: &str) {
        let Some(store) = try_cache() else {
            return;
        };
        if let Err(e) = Cache::del(store, &Self::key(token_id)).await {
            tracing::warn!(token_id, error = %e, "移除在线会话失败");
        }
    }

    /// 读取单条在线会话。
    ///
    /// # 参数
    ///
    /// - `token_id`：访问令牌 `jti`。
    ///
    /// # 返回值
    ///
    /// 存在返回 `Some`，不存在或已过期返回 `None`。
    ///
    /// # 异常情况
    ///
    /// 缓存未初始化或读取失败时返回错误。
    pub async fn find(token_id: &str) -> std::result::Result<Option<OnlineSession>, Error> {
        let Some(store) = try_cache() else {
            return Err(Error::Msg("缓存未初始化，无法查询在线会话".to_string()));
        };
        Cache::get(store, &Self::key(token_id)).await
    }

    /// 列出全部在线会话（按登录时间倒序）。
    ///
    /// # 返回值
    ///
    /// 在线会话列表；缓存未初始化时返回空列表。
    ///
    /// # 异常情况
    ///
    /// 枚举键失败时返回错误。个别键读取失败只记告警并跳过，不影响整体列表。
    pub async fn list_all() -> std::result::Result<Vec<OnlineSession>, Error> {
        let Some(store) = try_cache() else {
            return Ok(Vec::new());
        };
        let pattern = format!("{}*", constants::CACHE_KEY_PREFIX_ONLINE);
        let keys = Cache::keys(store, &pattern).await?;
        let mut sessions = Vec::with_capacity(keys.len());
        for key in keys {
            match Cache::get::<OnlineSession>(store, &key).await {
                Ok(Some(session)) => sessions.push(session),
                Ok(None) => {}
                Err(e) => tracing::warn!(key, error = %e, "读取在线会话失败，已跳过"),
            }
        }
        sessions.sort_by_key(|b| std::cmp::Reverse(b.login_time));
        Ok(sessions)
    }

    /// 强制下线：移除在线会话并把令牌写入黑名单（立即失效）。
    ///
    /// 黑名单键格式与框架 [`JWT::is_blacklisted`] 保持一致，TTL 取令牌剩余
    /// 有效期，从而认证层的黑名单检查可立即识别。
    ///
    /// # 参数
    ///
    /// - `session`：待下线的在线会话。
    ///
    /// # 异常情况
    ///
    /// 缓存未初始化或写入失败时返回错误。
    pub async fn force_logout(session: &OnlineSession) -> std::result::Result<(), Error> {
        let Some(store) = try_cache() else {
            return Err(Error::Msg("缓存未初始化，无法强制下线".to_string()));
        };
        Cache::del(store, &Self::key(&session.token_id)).await?;

        let now = chrono::Utc::now().timestamp();
        let ttl = (session.expire_at - now).max(1) as u64;
        let blacklist_key = format!(
            "{}{}",
            constants::CACHE_KEY_PREFIX_TOKEN_BLACKLIST,
            session.token_id
        );
        Cache::set_ex(store, &blacklist_key, &serde_json::json!(true), ttl).await
    }

    /// 生成在线会话缓存键。
    ///
    /// # 参数
    ///
    /// - `token_id`：访问令牌 `jti`。
    ///
    /// # 返回值
    ///
    /// 形如 `online:token:{jti}` 的缓存键。
    fn key(token_id: &str) -> String {
        format!("{}{}", constants::CACHE_KEY_PREFIX_ONLINE, token_id)
    }
}

/// 强制下线指定用户的全部在线令牌（权限被收回 / 停用 / 删除用户时调用）。
///
/// 枚举全部在线会话，对 `user_id` 命中的每个会话执行
/// [`OnlineSessionModel::force_logout`]，从而该用户所有已签发但仍有效的访问令牌
/// 在下次请求时即被黑名单拦截。
///
/// # 参数
///
/// - `user_id`：目标用户 ID（令牌 `sub`）。
///
/// # 返回值
///
/// 实际下线的令牌数量。缓存未初始化时返回 0（按「无在线会话」处理）。
/// 单个令牌拉黑失败只记告警，不中断其余令牌处理。
pub(crate) async fn force_logout_user(user_id: &str) -> usize {
    let sessions = match OnlineSessionModel::list_all().await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::warn!(user_id, error = %e, "枚举在线会话失败，跳过强制下线");
            return 0;
        }
    };
    let mut kicked = 0usize;
    for session in sessions.into_iter().filter(|s| s.user_id == user_id) {
        match OnlineSessionModel::force_logout(&session).await {
            Ok(()) => kicked += 1,
            Err(e) => tracing::warn!(
                user_id,
                token_id = %session.token_id,
                error = %e,
                "强制下线令牌失败"
            ),
        }
    }
    kicked
}

/// 强制下线除指定用户名外的全部在线用户（菜单 / 角色权限全局变更时调用）。
///
/// 菜单、角色菜单授权等变更会影响一批用户，而 JWT 权限在签发时即固化在令牌中，
/// 无法按权限点批量撤回；因此把受影响的在线令牌统一拉黑，令其重新登录获取新权限。
/// 通常排除内置超级管理员（权限短路、不受菜单变更影响），避免管理员改完权限后
/// 把自己踢下线。
///
/// # 参数
///
/// - `exclude_usernames`：需要保留在线状态的用户名集合（如 `["admin"]`）。
///
/// # 返回值
///
/// 实际下线的令牌数量。缓存未初始化时返回 0。
pub(crate) async fn force_logout_all_except(exclude_usernames: &[&str]) -> usize {
    let sessions = match OnlineSessionModel::list_all().await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::warn!(error = %e, "枚举在线会话失败，跳过批量强制下线");
            return 0;
        }
    };
    let mut kicked = 0usize;
    for session in sessions
        .into_iter()
        .filter(|s| !exclude_usernames.contains(&s.username.as_str()))
    {
        match OnlineSessionModel::force_logout(&session).await {
            Ok(()) => kicked += 1,
            Err(e) => tracing::warn!(
                user_id = %session.user_id,
                token_id = %session.token_id,
                error = %e,
                "强制下线令牌失败"
            ),
        }
    }
    kicked
}
