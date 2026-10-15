import { del, get, type PageData } from '@/api/request';

/** 操作日志 */
export interface LogRes {
  /** 日志 id */
  id: string;
  /** 请求链路 id */
  request_id: string;
  /** 操作用户 id */
  user_id: string;
  /** 操作用户名 */
  username: string;
  /** 操作描述 */
  operation: string;
  /** 目标资源类型 */
  target_type: string;
  /** 目标资源 id */
  target_id: string;
  /** 客户端 ip */
  ip: string;
  /** User-Agent */
  user_agent: string;
  /** 耗时（毫秒） */
  duration_ms: number;
  /** 请求路径 */
  request_path: string;
  /** 请求方法 */
  request_method: string;
  /** 响应状态码 */
  status_code: number;
  /** 记录时间 */
  created_at: string;
}

/** 操作日志详情（含请求参数等扩展信息） */
export interface LogDetailRes extends LogRes {
  /** 详情描述 */
  detail: string;
  /** 请求参数（脱敏后的任意 JSON） */
  request_params: unknown;
  /** 响应数据（脱敏后的任意 JSON） */
  response_data: unknown;
}

/** 日志分页查询参数 */
export type LogPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 操作用户名（模糊） */
  username?: string;
  /** 操作描述（模糊） */
  operation?: string;
  /** 请求路径（模糊） */
  request_path?: string;
  /** 响应状态码 */
  status_code?: number;
  /** 开始时间 */
  start_time?: string;
  /** 结束时间 */
  end_time?: string;
};

/** 分页查询操作日志 */
export function pageLogs(query: LogPageQuery): Promise<PageData<LogRes>> {
  return get<PageData<LogRes>>('/api/support/log/page', query);
}

/** 查询操作日志详情 */
export function getLog(id: string): Promise<LogDetailRes> {
  return get<LogDetailRes>(`/api/support/log/${id}`);
}

/** 清理指定天数之前的日志 */
export function cleanLogs(days: number): Promise<{ removed: number }> {
  return del<{ removed: number }>('/api/support/log/clean', { days });
}

/**
 * 通用日志行（审计 / 认证 / 安全 / 改密日志统一按整行 JSON 渲染）。
 *
 * 这四类日志的字段与操作日志不同，除公共字段外其余字段不确定，
 * 故用索引签名兜底，展示侧统一做空值处理。
 */
export interface RawLogRow {
  /** 日志 id */
  id: string;
  /** 请求链路 id */
  request_id?: string;
  /** 用户名 */
  username?: string;
  /** 记录时间 */
  created_at?: string;
  /** 客户端 ip */
  ip?: string;
  /** 其余随日志类型变化的字段 */
  [key: string]: unknown;
}

/** 通用日志分页查询参数（审计 / 认证 / 安全 / 改密日志共用） */
export type RawLogPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 用户名（模糊） */
  username?: string;
  /** 开始时间 */
  start_time?: string;
  /** 结束时间 */
  end_time?: string;
};

/** 分页查询审计日志 */
export function pageAuditLogs(query: RawLogPageQuery): Promise<PageData<RawLogRow>> {
  return get<PageData<RawLogRow>>('/api/support/log/audit/page', query);
}

/** 分页查询认证（登录）日志 */
export function pageAuthLogs(query: RawLogPageQuery): Promise<PageData<RawLogRow>> {
  return get<PageData<RawLogRow>>('/api/support/log/auth/page', query);
}

/** 分页查询安全日志 */
export function pageSecurityLogs(query: RawLogPageQuery): Promise<PageData<RawLogRow>> {
  return get<PageData<RawLogRow>>('/api/support/log/security/page', query);
}

/** 分页查询改密日志 */
export function pagePasswordLogs(query: RawLogPageQuery): Promise<PageData<RawLogRow>> {
  return get<PageData<RawLogRow>>('/api/support/log/password/page', query);
}

/** 日志模块接口集合 */
export const logApi = {
  pageLogs,
  getLog,
  cleanLogs,
  pageAuditLogs,
  pageAuthLogs,
  pageSecurityLogs,
  pagePasswordLogs,
};
