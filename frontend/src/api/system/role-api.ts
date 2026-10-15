import { del, get, post, put, type PageData } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 角色列表项 */
export interface RoleRes {
  /** 角色 id */
  id: string;
  /** 角色编码 */
  role_code: string;
  /** 角色名称 */
  role_name: string;
  /** 角色描述 */
  description: string;
  /** 状态 */
  status: StatusValue;
  /** 排序值 */
  sort_order: number;
  /** 备注 */
  remark: string;
  /** 创建时间 */
  created_at: string;
}

/** 角色分页查询参数 */
export type RolePageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 角色名称（模糊） */
  role_name?: string;
  /** 角色编码（模糊） */
  role_code?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 新增/编辑角色参数 */
export type RoleSaveParams = {
  /** 角色编码 */
  role_code: string;
  /** 角色名称 */
  role_name: string;
  /** 角色描述 */
  description?: string;
  /** 状态 */
  status?: StatusValue;
  /** 排序值 */
  sort_order?: number;
  /** 备注 */
  remark?: string;
};

/** 分页查询角色 */
export function pageRoles(query: RolePageQuery): Promise<PageData<RoleRes>> {
  return get<PageData<RoleRes>>('/api/system/role/page', query);
}

/** 查询全部启用角色，供下拉选择 */
export function listRoles(): Promise<RoleRes[]> {
  return get<RoleRes[]>('/api/system/role/list');
}

/** 查询角色详情 */
export function getRole(id: string): Promise<RoleRes> {
  return get<RoleRes>(`/api/system/role/${id}`);
}

/** 新增角色 */
export function createRole(data: RoleSaveParams): Promise<null> {
  return post<null>('/api/system/role', data);
}

/** 编辑角色 */
export function updateRole(id: string, data: RoleSaveParams): Promise<null> {
  return put<null>(`/api/system/role/${id}`, data);
}

/** 删除角色 */
export function deleteRole(id: string): Promise<null> {
  return del<null>(`/api/system/role/${id}`);
}

/** 查询角色已授权的菜单 id 列表 */
export function getRoleMenus(id: string): Promise<string[]> {
  return get<string[]>(`/api/system/role/${id}/menus`);
}

/** 保存角色菜单授权 */
export function updateRoleMenus(id: string, menuIds: string[]): Promise<null> {
  return put<null>(`/api/system/role/${id}/menus`, { menu_ids: menuIds });
}

/** 角色下的用户列表项 */
export interface RoleUserRes {
  /** 用户 id */
  id: string;
  /** 用户名 */
  username: string;
  /** 真实姓名 */
  real_name: string;
  /** 手机号 */
  mobile: string;
  /** 状态 */
  status: StatusValue;
  /** 该用户拥有的角色名称列表 */
  roles: string[];
}

/** 分页查询持有该角色的用户 */
export function getRoleUsers(
  id: string,
  query: { page: number; page_size: number },
): Promise<PageData<RoleUserRes>> {
  return get<PageData<RoleUserRes>>(`/api/system/role/${id}/users`, query);
}

/** 角色模块接口集合 */
export const roleApi = {
  pageRoles,
  listRoles,
  getRole,
  createRole,
  updateRole,
  deleteRole,
  getRoleMenus,
  updateRoleMenus,
  getRoleUsers,
};
