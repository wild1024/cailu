import { del, get, type PageData } from '@/api/request';

/** 在线用户列表项（对应后端 `OnlineRes`） */
export interface OnlineRes {
  /** 访问令牌唯一标识（强制下线时作为定位键） */
  token_id: string;
  /** 用户 id */
  user_id: string;
  /** 用户名 */
  username: string;
  /** 客户端 ip */
  ip: string;
  /** 登录地点 */
  location: string;
  /** 浏览器类型 */
  browser: string;
  /** 操作系统类型 */
  os_type: string;
  /** 登录时间 */
  login_time: string;
  /** 令牌到期时间 */
  expire_at: string;
}

/** 在线用户分页查询参数 */
export type OnlinePageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 用户名（模糊） */
  username?: string;
};

/** 分页查询在线用户 */
export function pageOnline(query: OnlinePageQuery): Promise<PageData<OnlineRes>> {
  return get<PageData<OnlineRes>>('/api/support/online/page', query);
}

/** 强制指定在线会话下线 */
export function forceLogout(tokenId: string): Promise<null> {
  return del<null>(`/api/support/online/${tokenId}`);
}

/** 在线用户模块接口集合 */
export const onlineApi = {
  pageOnline,
  forceLogout,
};
