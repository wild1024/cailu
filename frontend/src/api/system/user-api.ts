import { del, get, patch, post, put, type PageData } from '@/api/request';

/** 通用启用状态：0=停用 1=正常 */
export type StatusValue = 0 | 1;

/** 用户列表项 */
export interface UserRes {
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
  /** 所属部门名称 */
  dept_name: string;
  /** 状态 */
  status: StatusValue;
  /** 是否被锁定 */
  is_locked: boolean;
  /** 最近登录时间 */
  last_login_at: string | null;
  /** 创建时间 */
  created_at: string;
  /** 关联角色简要列表 */
  roles: UserRoleBrief[];
}

/** 用户关联角色的简要信息（列表 / 详情聚合下发） */
export interface UserRoleBrief {
  /** 角色 id */
  role_id: string;
  /** 角色编码 */
  role_code: string;
  /** 角色名称 */
  role_name: string;
}

/** 用户详情（比列表多 remark） */
export interface UserDetailRes extends UserRes {
  /** 备注 */
  remark: string;
}

/** 用户分页查询参数 */
export type UserPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 用户名（模糊） */
  username?: string;
  /** 真实姓名（模糊） */
  real_name?: string;
  /** 状态 */
  status?: StatusValue;
  /** 部门 id */
  dept_id?: string;
  /** 是否包含所选部门的子部门用户（true 时按部门子树过滤） */
  include_children?: boolean;
};

/** 新增用户参数 */
export type UserCreateParams = {
  /** 用户名 */
  username: string;
  /** 初始密码 */
  password: string;
  /** 真实姓名 */
  real_name?: string;
  /** 邮箱 */
  email?: string;
  /** 手机号 */
  mobile?: string;
  /** 所属部门 id */
  dept_id?: string;
  /** 状态 */
  status?: StatusValue;
  /** 备注 */
  remark?: string;
  /** 角色 id 列表（创建用户同时分配角色，可为空） */
  role_ids?: string[];
};

/** 编辑用户参数 */
export type UserUpdateParams = {
  /** 真实姓名 */
  real_name?: string;
  /** 邮箱 */
  email?: string;
  /** 手机号 */
  mobile?: string;
  /** 所属部门 id */
  dept_id?: string;
  /** 状态 */
  status?: StatusValue;
  /** 备注 */
  remark?: string;
};

/** 分页查询用户 */
export function pageUsers(query: UserPageQuery): Promise<PageData<UserRes>> {
  return get<PageData<UserRes>>('/api/system/user/page', query);
}

/** 查询用户详情 */
export function getUser(id: string): Promise<UserDetailRes> {
  return get<UserDetailRes>(`/api/system/user/${id}`);
}

/** 新增用户 */
export function createUser(data: UserCreateParams): Promise<null> {
  return post<null>('/api/system/user', data);
}

/** 编辑用户 */
export function updateUser(id: string, data: UserUpdateParams): Promise<null> {
  return put<null>(`/api/system/user/${id}`, data);
}

/** 删除用户 */
export function deleteUser(id: string): Promise<null> {
  return del<null>(`/api/system/user/${id}`);
}

/** 修改用户状态 */
export function changeUserStatus(id: string, status: StatusValue): Promise<null> {
  return patch<null>(`/api/system/user/${id}/status`, { status });
}

/** 重置用户密码 */
export function resetUserPassword(id: string, password: string): Promise<null> {
  return patch<null>(`/api/system/user/${id}/password`, { password });
}

/** 查询用户已分配的角色 id 列表 */
export function getUserRoles(id: string): Promise<string[]> {
  return get<string[]>(`/api/system/user/${id}/roles`);
}

/** 分配用户角色 */
export function updateUserRoles(id: string, roleIds: string[]): Promise<null> {
  return put<null>(`/api/system/user/${id}/roles`, { role_ids: roleIds });
}

/** 用户模块接口集合 */
export const userApi = {
  pageUsers,
  getUser,
  createUser,
  updateUser,
  deleteUser,
  changeUserStatus,
  resetUserPassword,
  getUserRoles,
  updateUserRoles,
};
