import { get, post, put, type PageData } from '@/api/request';

/** 菜单类型：1=目录 2=菜单 3=按钮 */
export type MenuType = 1 | 2 | 3;

/** 菜单树节点（`/api/system/auth/menus` 返回，含按钮类型权限点） */
export interface MenuNode {
  /** 菜单 id */
  id: string;
  /** 菜单名称（唯一标识，如 system:user） */
  name: string;
  /** 菜单标题（界面展示） */
  title: string;
  /** 路由路径 */
  path: string;
  /** 前端组件路径 */
  component: string;
  /** 图标名 */
  icon: string;
  /** 菜单类型 */
  menu_type: MenuType;
  /** 权限码 */
  auth_code: string;
  /** 上级菜单 id，顶级为空 */
  parent_id: string | null;
  /** 排序值 */
  sort_order: number;
  /** 是否在导航中显示 */
  is_visible: boolean;
  /** 子节点 */
  children: MenuNode[];
}

/** 登录请求体 */
export type LoginParams = {
  /** 用户名 */
  username: string;
  /** 密码 */
  password: string;
};

/** 登录返回的用户摘要 */
export interface LoginUserInfo {
  /** 用户 id */
  id: string;
  /** 用户名 */
  username: string;
  /** 真实姓名 */
  real_name: string;
}

/** 登录响应体 */
export interface LoginRes {
  /** 访问令牌 */
  access_token: string;
  /** 刷新令牌 */
  refresh_token: string;
  /** 令牌类型，固定 Bearer */
  token_type: string;
  /** 访问令牌有效期（秒） */
  expires_in: number;
  /** 登录用户摘要 */
  user: LoginUserInfo;
}

/** 刷新令牌响应体（不含用户信息） */
export interface RefreshRes {
  /** 访问令牌 */
  access_token: string;
  /** 刷新令牌 */
  refresh_token: string;
  /** 令牌类型，固定 Bearer */
  token_type: string;
  /** 访问令牌有效期（秒） */
  expires_in: number;
}

/** 当前登录用户详情（`/me`） */
export interface CurrentUserRes {
  /** 用户 id */
  id: string;
  /** 用户名 */
  username: string;
  /** 真实姓名 */
  real_name: string;
  /** 邮箱 */
  email: string;
  /** 手机号 */
  mobile: string;
  /** 所属部门 id */
  dept_id: string | null;
  /** 角色编码列表 */
  roles: string[];
  /** 权限码列表，超级管理员为 ["*:*:*"] */
  permissions: string[];
}

/** 登录（认证）日志行：后端以 `to_jsonb` 下发整行，字段可能随表结构变化 */
export interface AuthLogRow {
  /** 日志 id */
  id?: string;
  /** 用户名 */
  username?: string | null;
  /** 认证类型：LOGIN / LOGOUT / REFRESH_TOKEN */
  auth_type?: string | null;
  /** 认证状态：SUCCESS / FAIL */
  auth_status?: string | null;
  /** 客户端 IP */
  ip?: string | null;
  /** 浏览器 User-Agent */
  user_agent?: string | null;
  /** 失败原因 */
  error_message?: string | null;
  /** 登录耗时（毫秒） */
  login_duration?: number | null;
  /** 记录时间 */
  created_at?: string;
}

/** 更新个人资料请求体（仅提交需要变更的字段） */
export interface UpdateProfileReq {
  /** 真实姓名 */
  real_name?: string;
  /** 邮箱 */
  email?: string;
  /** 手机号 */
  mobile?: string;
}

/** 修改本人密码请求体 */
export interface ChangePasswordReq {
  /** 原密码 */
  old_password: string;
  /** 新密码 */
  new_password: string;
}

/** 登录（免登录接口） */
export function login(params: LoginParams): Promise<LoginRes> {
  return post<LoginRes>('/api/system/auth/login', params);
}

/** 刷新令牌（免登录接口，401 时由 request.ts 自动调用） */
export function refresh(refreshToken: string): Promise<RefreshRes> {
  return post<RefreshRes>('/api/system/auth/refresh', { refresh_token: refreshToken });
}

/** 登出 */
export function logout(): Promise<null> {
  return post<null>('/api/system/auth/logout');
}

/** 查询当前登录用户信息（含角色编码与权限码） */
export function fetchMe(): Promise<CurrentUserRes> {
  return get<CurrentUserRes>('/api/system/auth/me');
}

/** 查询当前登录用户可见的菜单树 */
export function fetchMenus(): Promise<MenuNode[]> {
  return get<MenuNode[]>('/api/system/auth/menus');
}

/** 更新当前登录用户的个人资料（姓名 / 邮箱 / 手机号） */
export function updateProfile(data: UpdateProfileReq): Promise<null> {
  return put<null>('/api/system/auth/profile', data);
}

/** 修改当前登录用户的密码（需校验原密码） */
export function changePassword(data: ChangePasswordReq): Promise<null> {
  return put<null>('/api/system/auth/password', data);
}

/** 查询当前登录用户自己的登录（认证）日志 */
export function fetchMyLoginLogs(params: {
  /** 页码，从 1 开始 */
  page?: number;
  /** 每页条数 */
  page_size?: number;
}): Promise<PageData<AuthLogRow>> {
  return get<PageData<AuthLogRow>>('/api/system/auth/logs', params);
}

/** 认证模块接口集合 */
export const authApi = {
  login,
  refresh,
  logout,
  fetchMe,
  fetchMenus,
  updateProfile,
  changePassword,
  fetchMyLoginLogs,
};
