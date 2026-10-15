import { get } from '@/api/request';

/** 指标卡片的主题色键，对应前端预设色 */
export type CardTheme = 'blue' | 'green' | 'red' | 'orange' | 'purple' | 'cyan';

/** 单条指标卡片 */
export interface MetricCard {
  /** 指标键（前端据此判断特殊单位） */
  key: string;
  /** 指标名称 */
  label: string;
  /** 主数值 */
  value: number;
  /** 附加数值（失败率的分母 / 任务的启用数等） */
  extra: number;
  /** 单位后缀（% / ms / 空串） */
  unit: string;
  /** TDesign 图标名 */
  icon: string;
  /** 主题色键 */
  theme: CardTheme;
  /** 口径说明 */
  tip: string;
}

/** 趋势图的一条折线 */
export interface TrendLine {
  /** 折线名称 */
  name: string;
  /** 与 dates 等长的取值，空缺日为 null */
  data: (number | null)[];
}

/** 趋势图数据 */
export interface TrendData {
  /** X 轴日期标签（MM-DD） */
  dates: string[];
  /** 折线集合 */
  lines: TrendLine[];
}

/** 工作台概览聚合响应 */
export interface OverviewRes {
  /** 指标卡片列表 */
  metrics: MetricCard[];
  /** 登录成功 / 失败趋势 */
  login_trend: TrendData;
  /** 登录平均耗时趋势 */
  latency_trend: TrendData;
  /** 趋势窗口天数（含当天） */
  trend_days: number;
}

/**
 * 工作台概览：一次请求取回指标卡片与两张趋势图。
 */
export function fetchOverview(): Promise<OverviewRes> {
  // 路径必须写全（含 /api 前缀）：baseURL 留空是项目约定，
  // 漏写前缀会被 Vite 的 SPA fallback 回成 index.html，表现为「报表永远不显示」。
  return get<OverviewRes>('/api/support/dashboard/overview');
}
