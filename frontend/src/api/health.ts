import { get } from '@/api/request';

/** 健康检查响应体（对应后端 `support::health::HealthRes`） */
export interface HealthRes {
  /** 服务状态，固定为 `ok` */
  status: string;
  /** 应用名称 */
  app: string;
  /** 当前生效的配置 profile */
  profile: string;
  /** 服务端时间（RFC 3339） */
  time: string;
}

/** 查询后端健康状态，用于链路连通性校验 */
export function fetchHealth(): Promise<HealthRes> {
  return get<HealthRes>('/health');
}
