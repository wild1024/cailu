import { del, get, patch, post, put, type PageData } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 定时任务 */
export interface JobRes {
  /** 任务 id */
  id: string;
  /** 任务名称 */
  job_name: string;
  /** 任务分组 */
  job_group: string;
  /** cron 表达式 */
  cron_expr: string;
  /** 调用目标（如 Rust 函数路径） */
  invoke_target: string;
  /** 调用参数 */
  params: string;
  /** 状态 */
  status: StatusValue;
  /** 上次执行时间 */
  last_run_at: string | null;
  /** 下次执行时间 */
  next_run_at: string | null;
  /** 备注 */
  remark: string;
  /** 创建时间 */
  created_at: string;
}

/** 定时任务执行日志 */
export interface JobLogRes {
  /** 日志 id */
  id: string;
  /** 任务 id */
  job_id: string;
  /** 任务名称 */
  job_name: string;
  /** 调用目标 */
  invoke_target: string;
  /** 执行状态：1=成功 0=失败 */
  status: StatusValue;
  /** 执行消息 */
  message: string;
  /** 异常信息 */
  error: string;
  /** 开始时间 */
  started_at: string;
  /** 结束时间 */
  finished_at: string;
  /** 耗时（毫秒） */
  duration_ms: number;
  /** 记录时间 */
  created_at: string;
}

/** 定时任务分页查询参数 */
export type JobPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 任务名称（模糊） */
  job_name?: string;
  /** 任务分组 */
  job_group?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 定时任务新增/编辑参数 */
export type JobSaveParams = {
  /** 任务名称 */
  job_name: string;
  /** 任务分组 */
  job_group?: string;
  /** cron 表达式 */
  cron_expr: string;
  /** 调用目标 */
  invoke_target: string;
  /** 调用参数 */
  params?: string;
  /** 状态 */
  status?: StatusValue;
  /** 备注 */
  remark?: string;
};

/** 任务执行日志分页查询参数 */
export type JobLogPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 任务 id */
  job_id?: string;
  /** 任务名称（模糊） */
  job_name?: string;
  /** 执行状态 */
  status?: StatusValue;
};

/** 分页查询定时任务 */
export function pageJobs(query: JobPageQuery): Promise<PageData<JobRes>> {
  return get<PageData<JobRes>>('/api/support/job/page', query);
}

/** 查询定时任务详情 */
export function getJob(id: string): Promise<JobRes> {
  return get<JobRes>(`/api/support/job/${id}`);
}

/** 新增定时任务 */
export function createJob(data: JobSaveParams): Promise<null> {
  return post<null>('/api/support/job', data);
}

/** 编辑定时任务 */
export function updateJob(id: string, data: JobSaveParams): Promise<null> {
  return put<null>(`/api/support/job/${id}`, data);
}

/** 删除定时任务 */
export function deleteJob(id: string): Promise<null> {
  return del<null>(`/api/support/job/${id}`);
}

/** 启停定时任务 */
export function changeJobStatus(id: string, status: StatusValue): Promise<null> {
  return patch<null>(`/api/support/job/${id}/status`, { status });
}

/** 手动执行一次定时任务 */
export function triggerJob(id: string): Promise<null> {
  return post<null>(`/api/support/job/${id}/trigger`);
}

/** 分页查询任务执行日志 */
export function pageJobLogs(query: JobLogPageQuery): Promise<PageData<JobLogRes>> {
  return get<PageData<JobLogRes>>('/api/support/job/log/page', query);
}

/** 查询任务执行日志详情 */
export function getJobLog(id: string): Promise<JobLogRes> {
  return get<JobLogRes>(`/api/support/job/log/${id}`);
}

/** 定时任务模块接口集合 */
export const jobApi = {
  pageJobs,
  getJob,
  createJob,
  updateJob,
  deleteJob,
  changeJobStatus,
  triggerJob,
  pageJobLogs,
  getJobLog,
};
