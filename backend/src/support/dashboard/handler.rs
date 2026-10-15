//! 工作台概览接口：首页指标卡片与趋势图数据。
//!
//! 概览是**只读的旁路读接口**：不写任何表、不加锁，全部走聚合查询；在线用户数来自缓存
//! （见 [`crate::support::online::model`]），失败只记日志并降级为 0，不影响首页可用。

use alun::prelude::*;

use crate::shared::constants;
use crate::support::dashboard::dto::OverviewRes;
use crate::support::dashboard::model;
use crate::support::online::model::OnlineSessionModel;
use crate::utils;

// ==================== 接口 ====================

/// 工作台概览（指标卡片 + 两张趋势图）。
///
/// 整页结果按固定键做 **60s 短 TTL 缓存**：概览由十余条全表聚合组成，而工作台
/// 是登录后落地页、刷新频繁，缓存能显著削减重复聚合；60s 又足以让「今日在线 /
/// 登录次数」这类指标保持实时感（缓存不可用时自动回退为每次直查）。
///
/// # 返回值
///
/// 成功返回聚合结果 [`OverviewRes`]；任一统计 SQL 异常时返回 500。
///
/// # 异常情况
///
/// - 500：数据库聚合查询失败；
/// - 500：读取在线会话（缓存）失败（此时其余指标照常返回，在线数降为 0）。
#[alun::permission(
    path = "/support/dashboard/overview",
    method = "GET",
    permission = "support:dashboard:list"
)]
#[alun::get("/support/dashboard/overview")]
pub async fn overview() -> ResResult<OverviewRes> {
    if let Some(store) = try_cache() {
        if let Some(cached) = Cache::get::<OverviewRes>(store, constants::CACHE_KEY_DASHBOARD_METRICS)
            .await
            .ok()
            .flatten()
        {
            return Ok(Res::ok(cached));
        }
    }

    let overview = load_overview().await?;

    if let Some(store) = try_cache() {
        if let Err(e) = Cache::set_ex(
            store,
            constants::CACHE_KEY_DASHBOARD_METRICS,
            &overview,
            constants::CACHE_DASHBOARD_TTL_SECS,
        )
        .await
        {
            tracing::warn!(error = %e, "写入工作台概览缓存失败");
        }
    }

    Ok(Res::ok(overview))
}

/// 实时计算工作台概览（不走缓存）。
///
/// # 返回值
///
/// 成功返回 [`OverviewRes`]。
///
/// # 异常情况
///
/// 数据库聚合查询失败时返回 500。
async fn load_overview() -> std::result::Result<OverviewRes, ApiError> {
    let metrics = model::load_metrics()
        .await
        .map_err(|e| utils::db_error(e, "查询工作台指标"))?;

    // 在线会话存在缓存（未启用缓存时为进程内实现），读取失败不影响主体指标
    let online_users = match OnlineSessionModel::list_all().await {
        Ok(sessions) => sessions.len(),
        Err(e) => {
            tracing::warn!(error = %e, "读取在线会话失败，工作台在线用户数按 0 展示");
            0
        }
    };
    let metrics_cards = model::build_metric_cards(&metrics, online_users);

    let login_rows = model::load_login_trend()
        .await
        .map_err(|e| utils::db_error(e, "查询登录趋势"))?;
    let latency_rows = model::load_latency_trend()
        .await
        .map_err(|e| utils::db_error(e, "查询登录耗时趋势"))?;

    Ok(OverviewRes {
        metrics: metrics_cards,
        login_trend: model::to_login_trend_res(&login_rows),
        latency_trend: model::to_latency_trend_res(&latency_rows),
        trend_days: model::TREND_DAYS,
    })
}