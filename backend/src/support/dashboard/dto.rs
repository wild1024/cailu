//! 工作台概览的请求与响应 DTO。
//!
//! 字段名与前端契约 `frontend/src/api/dashboard-api.ts` 严格对齐。
//!
//! 概览数据一次性下发三段：
//!
//! - [`MetricCardRes`]：指标卡片（图标、主题色与口径说明随数据一起下发，前端只负责渲染）；
//! - [`TrendRes`]：趋势图（统一为「X 轴日期 + 若干条折线」的二维结构，两张图共用同一形状，
//!   便于前端复用同一个 ECharts 配置）；
//! - [`OverviewRes`]：整页聚合结果。

use serde::{Deserialize, Serialize};

/// 单条指标卡片。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCardRes {
    /// 指标键（前端据此做动画或判断特殊单位，如 `fail_rate` 带百分号）。
    pub key: String,
    /// 指标名称（如「用户总数」）。
    pub label: String,
    /// 主数值。
    pub value: i64,
    /// 附加数值（语义随指标而定：失败率的分母、任务数的启用数等；无意义时为 0）。
    pub extra: i64,
    /// 单位后缀（如 `%` / `ms` / 空串）。
    pub unit: String,
    /// 图标名（TDesign 图标库）。
    pub icon: String,
    /// 主题色键，前端映射到预设色（`blue` / `green` / `red` / `orange` / `purple` / `cyan`）。
    pub theme: String,
    /// 口径说明（展示在卡片底部，避免前端硬编码统计规则）。
    pub tip: String,
}

/// 趋势图的一条折线。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendLineRes {
    /// 折线名称（图例显示）。
    pub name: String,
    /// 与 `dates` 等长的取值，空缺日以 `null` 占位（前端画断点）。
    pub data: Vec<Option<i64>>,
}

/// 趋势图数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendRes {
    /// X 轴日期标签（`MM-DD`），与每条折线等长。
    pub dates: Vec<String>,
    /// 折线集合（登录趋势为成功 / 失败两条，耗时趋势为平均一条）。
    pub lines: Vec<TrendLineRes>,
}

/// 工作台概览聚合响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewRes {
    /// 指标卡片列表。
    pub metrics: Vec<MetricCardRes>,
    /// 趋势图一：近 N 天登录成功 / 失败次数。
    pub login_trend: TrendRes,
    /// 趋势图二：近 N 天登录平均耗时（毫秒）。
    pub latency_trend: TrendRes,
    /// 趋势窗口天数（含当天），供前端兜底文案使用。
    pub trend_days: i64,
}
