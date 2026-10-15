import { del, get, post, put } from '@/api/request';
import type { MenuNode, MenuType } from '@/api/system/auth-api';
import type { StatusValue } from '@/api/system/user-api';

/** 菜单明细（含按钮类型权限点） */
export interface MenuRes {
  /** 菜单 id */
  id: string;
  /** 菜单名称（唯一标识） */
  name: string;
  /** 菜单类型：1=目录 2=菜单 3=按钮 */
  menu_type: MenuType;
  /** 权限码 */
  auth_code: string;
  /** 路由路径 */
  path: string;
  /** 上级菜单 id */
  parent_id: string | null;
  /** 前端组件路径 */
  component: string;
  /** 排序值 */
  sort_order: number;
  /** 状态 */
  status: StatusValue;
  /** 图标名 */
  icon: string;
  /** 菜单标题 */
  title: string;
  /** 是否外链 */
  is_frame: boolean;
  /** 是否缓存页面 */
  is_cache: boolean;
  /** 是否在导航中显示 */
  is_visible: boolean;
  /** 所属模块编码 */
  module_code: string;
}

/** 菜单树查询参数 */
export type MenuTreeQuery = {
  /** 菜单名称（模糊） */
  name?: string;
  /** 状态 */
  status?: StatusValue;
  /** 模块编码 */
  module_code?: string;
};

/** 新增/编辑菜单参数 */
export type MenuSaveParams = Omit<MenuRes, 'id'>;

/** 查询菜单树（含按钮） */
export function getMenuTree(query: MenuTreeQuery = {}): Promise<MenuNode[]> {
  return get<MenuNode[]>('/api/system/menu/tree', query);
}

/** 查询菜单详情 */
export function getMenu(id: string): Promise<MenuRes> {
  return get<MenuRes>(`/api/system/menu/${id}`);
}

/** 新增菜单 */
export function createMenu(data: MenuSaveParams): Promise<null> {
  return post<null>('/api/system/menu', data);
}

/** 编辑菜单 */
export function updateMenu(id: string, data: MenuSaveParams): Promise<null> {
  return put<null>(`/api/system/menu/${id}`, data);
}

/** 删除菜单 */
export function deleteMenu(id: string): Promise<null> {
  return del<null>(`/api/system/menu/${id}`);
}

/** 菜单模块接口集合 */
export const menuApi = {
  getMenuTree,
  getMenu,
  createMenu,
  updateMenu,
  deleteMenu,
};
