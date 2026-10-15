import { del, get, post, put } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 部门树节点 */
export interface DeptNode {
  /** 部门 id */
  id: string;
  /** 部门编码 */
  dept_code: string;
  /** 部门名称 */
  dept_name: string;
  /** 上级部门 id */
  parent_id: string | null;
  /** 负责人用户 id */
  leader_id: string | null;
  /** 负责人姓名（real_name 优先，回退 username；后端树查询时填充） */
  leader_name?: string | null;
  /** 联系电话 */
  phone: string;
  /** 邮箱 */
  email: string;
  /** 状态 */
  status: StatusValue;
  /** 排序值 */
  sort_order: number;
  /** 子部门 */
  children: DeptNode[];
}

/** 部门详情（不含 children） */
export type DeptDetailRes = Omit<DeptNode, 'children'>;

/** 部门树查询参数 */
export type DeptTreeQuery = {
  /** 部门名称（模糊） */
  dept_name?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 新增/编辑部门参数 */
export type DeptSaveParams = {
  /** 部门编码 */
  dept_code: string;
  /** 部门名称 */
  dept_name: string;
  /** 上级部门 id */
  parent_id?: string;
  /** 负责人用户 id */
  leader_id?: string;
  /** 联系电话 */
  phone?: string;
  /** 邮箱 */
  email?: string;
  /** 状态 */
  status?: StatusValue;
  /** 排序值 */
  sort_order?: number;
};

/** 查询部门树 */
export function getDeptTree(query: DeptTreeQuery = {}): Promise<DeptNode[]> {
  return get<DeptNode[]>('/api/system/dept/tree', query);
}

/** 查询部门详情 */
export function getDept(id: string): Promise<DeptDetailRes> {
  return get<DeptDetailRes>(`/api/system/dept/${id}`);
}

/** 新增部门 */
export function createDept(data: DeptSaveParams): Promise<null> {
  return post<null>('/api/system/dept', data);
}

/** 编辑部门 */
export function updateDept(id: string, data: DeptSaveParams): Promise<null> {
  return put<null>(`/api/system/dept/${id}`, data);
}

/** 删除部门 */
export function deleteDept(id: string): Promise<null> {
  return del<null>(`/api/system/dept/${id}`);
}

/** 部门模块接口集合 */
export const deptApi = {
  getDeptTree,
  getDept,
  createDept,
  updateDept,
  deleteDept,
};
