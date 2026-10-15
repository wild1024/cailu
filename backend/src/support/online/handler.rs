//! 在线用户处理器：在线列表与强制下线。
//!
//! 会话数据来自缓存（见 [`OnlineSessionModel`]），量级为「当前在线令牌数」，
//! 因此列表在内存中过滤与分页，避免为一处旁路统计引入数据库表。

use alun::prelude::*;

use crate::shared::error_code;
use crate::support::online::dto::{OnlineQuery, OnlineRes};
use crate::support::online::model::OnlineSessionModel;
use crate::utils;

/// 分页查询在线用户。
///
/// # 参数
///
/// - `query`：分页与过滤条件（用户名模糊匹配）。
///
/// # 返回值
///
/// 成功返回在线用户分页列表（按登录时间倒序）。
///
/// # 异常情况
///
/// - 503：缓存未初始化或枚举失败。
#[alun::permission(
    path = "/support/online/page",
    method = "GET",
    permission = "support:online:list"
)]
#[alun::get("/support/online/page")]
pub async fn page(Query(query): Query<OnlineQuery>) -> ResResult<PageData<Vec<OnlineRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    let sessions = OnlineSessionModel::list_all()
        .await
        .map_err(|e| cache_error(e, "枚举在线会话"))?;

    let keyword = query
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let matched: Vec<OnlineRes> = sessions
        .iter()
        .filter(|session| match keyword {
            Some(keyword) => session.username.contains(keyword),
            None => true,
        })
        .map(OnlineRes::from)
        .collect();

    let total = matched.len() as u64;
    let start = ((page - 1) * page_size) as usize;
    let list = matched
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    Ok(Res::page(list, total, page, page_size))
}

/// 强制下线指定在线会话。
///
/// 移除在线记录并把该令牌写入黑名单，令牌在剩余有效期内立即失效。
///
/// # 参数
///
/// - `token_id`：访问令牌 `jti`。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 404：会话不存在或已过期；
/// - 503：缓存未初始化或写入失败。
#[alun::permission(
    path = "/support/online/{token_id}",
    method = "DELETE",
    permission = "support:online:force"
)]
#[alun::delete("/support/online/{token_id}")]
pub async fn force_logout(Path(token_id): Path<String>) -> ResResult<()> {
    let session = OnlineSessionModel::find(&token_id)
        .await
        .map_err(|e| cache_error(e, "查询在线会话"))?
        .ok_or_else(|| {
            ApiError::new(
                404,
                error_code::ONLINE_SESSION_NOT_FOUND,
                "在线会话不存在或已过期",
            )
        })?;

    OnlineSessionModel::force_logout(&session)
        .await
        .map_err(|e| cache_error(e, "强制下线"))?;

    tracing::info!(token_id = %token_id, username = %session.username, "已强制下线在线用户");
    Ok(Res::ok_msg("已强制下线"))
}

/// 把缓存错误映射为统一的 503 服务不可用错误（对外隐藏细节，细节写入日志）。
///
/// # 参数
///
/// - `e`：缓存 / 框架错误。
/// - `ctx`：业务上下文描述，用于日志定位。
///
/// # 返回值
///
/// 统一响应体为 `code = ONLINE_CACHE_UNAVAILABLE` 的 [`ApiError`]。
fn cache_error(e: Error, ctx: &str) -> ApiError {
    tracing::error!(context = ctx, error = %e, "在线用户缓存操作失败");
    ApiError::new(
        503,
        error_code::ONLINE_CACHE_UNAVAILABLE,
        "在线用户功能暂不可用",
    )
}
