import { del, get, post, put, type PageData } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 参数配置项 */
export interface ConfigRes {
  /** 配置 id */
  id: string;
  /** 配置键 */
  config_key: string;
  /** 配置值 */
  config_value: string;
  /** 值类型（text/number/boolean/json 等） */
  config_type: string;
  /** 配置说明 */
  description: string;
  /** 配置分类 */
  category: string;
  /** 排序值 */
  sort_order: number;
  /** 配置级别，数值越小优先级越高 */
  config_level: number;
  /** 是否允许在界面修改 */
  is_modifiable: boolean;
  /** 是否加密存储 */
  is_encrypted: boolean;
  /** 是否需重启生效 */
  is_restart: boolean;
  /** 状态 */
  status: StatusValue;
  /** 引用值（其他配置键） */
  ref_value: string;
  /** 可选值列表 */
  select_value: string;
  /** 创建时间 */
  created_at: string;
}

/** 参数分页查询参数 */
export type ConfigPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 配置键（模糊） */
  config_key?: string;
  /** 值类型 */
  config_type?: string;
  /** 配置分类 */
  category?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 参数新增/编辑参数 */
export type ConfigSaveParams = {
  /** 配置键 */
  config_key: string;
  /** 配置值 */
  config_value?: string;
  /** 值类型 */
  config_type: string;
  /** 配置说明 */
  description?: string;
  /** 配置分类 */
  category?: string;
  /** 排序值 */
  sort_order?: number;
  /** 配置级别 */
  config_level?: number;
  /** 是否允许修改 */
  is_modifiable?: boolean;
  /** 是否加密存储 */
  is_encrypted?: boolean;
  /** 是否需重启生效 */
  is_restart?: boolean;
  /** 状态 */
  status?: StatusValue;
  /** 引用值 */
  ref_value?: string;
  /** 可选值列表 */
  select_value?: string;
};

/** 分页查询参数配置 */
export function pageConfigs(query: ConfigPageQuery): Promise<PageData<ConfigRes>> {
  return get<PageData<ConfigRes>>('/api/support/config/page', query);
}

/** 查询参数配置详情 */
export function getConfig(id: string): Promise<ConfigRes> {
  return get<ConfigRes>(`/api/support/config/${id}`);
}

/** 新增参数配置 */
export function createConfig(data: ConfigSaveParams): Promise<null> {
  return post<null>('/api/support/config', data);
}

/** 编辑参数配置 */
export function updateConfig(id: string, data: ConfigSaveParams): Promise<null> {
  return put<null>(`/api/support/config/${id}`, data);
}

/** 删除参数配置 */
export function deleteConfig(id: string): Promise<null> {
  return del<null>(`/api/support/config/${id}`);
}

/** 按配置键查询配置值 */
export function getConfigValue(configKey: string): Promise<string | null> {
  return get<string | null>(`/api/support/config/value/${configKey}`);
}

/**
 * 重载后端运行时参数快照。
 *
 * 用于多实例部署或直接改数据库的场景：强制本实例重新读取 `sys_config` 并广播，
 * 使订阅了参数的模块（如 IP 归属地解析开关）立即生效。
 *
 * @returns 重载后快照中的参数键列表（升序）
 */
export function reloadRuntimeConfig(): Promise<string[]> {
  return post<string[]>('/api/support/config/reload');
}

/** 参数配置模块接口集合 */
export const configApi = {
  pageConfigs,
  getConfig,
  createConfig,
  updateConfig,
  deleteConfig,
  getConfigValue,
  reloadRuntimeConfig,
};
