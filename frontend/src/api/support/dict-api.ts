import { del, get, post, put, type PageData } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 字典类型 */
export interface DictTypeRes {
  /** 字典类型 id */
  id: string;
  /** 字典编码 */
  dict_code: string;
  /** 字典名称 */
  dict_name: string;
  /** 描述 */
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

/** 字典项 */
export interface DictItemRes {
  /** 字典项 id */
  id: string;
  /** 所属字典类型 id */
  dict_type_id: string;
  /** 字典值 */
  item_value: string;
  /** 字典标签 */
  item_label: string;
  /** 描述 */
  description: string;
  /** 排序值 */
  sort_order: number;
  /** 是否默认项 */
  is_default: boolean;
  /** 状态 */
  status: StatusValue;
  /** 创建时间 */
  created_at: string;
}

/** 字典类型分页查询参数 */
export type DictTypePageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 字典编码（模糊） */
  dict_code?: string;
  /** 字典名称（模糊） */
  dict_name?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 字典类型新增/编辑参数 */
export type DictTypeSaveParams = {
  /** 字典编码 */
  dict_code: string;
  /** 字典名称 */
  dict_name: string;
  /** 描述 */
  description?: string;
  /** 状态 */
  status?: StatusValue;
  /** 排序值 */
  sort_order?: number;
  /** 备注 */
  remark?: string;
};

/** 字典项分页查询参数 */
export type DictItemPageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 所属字典类型 id */
  dict_type_id?: string;
  /** 字典编码 */
  dict_code?: string;
  /** 字典标签（模糊） */
  item_label?: string;
  /** 状态 */
  status?: StatusValue;
};

/** 字典项新增/编辑参数 */
export type DictItemSaveParams = {
  /** 所属字典类型 id */
  dict_type_id: string;
  /** 字典值 */
  item_value: string;
  /** 字典标签 */
  item_label: string;
  /** 描述 */
  description?: string;
  /** 排序值 */
  sort_order?: number;
  /** 是否默认项 */
  is_default?: boolean;
  /** 状态 */
  status?: StatusValue;
};

/** 分页查询字典类型 */
export function pageDictTypes(query: DictTypePageQuery): Promise<PageData<DictTypeRes>> {
  return get<PageData<DictTypeRes>>('/api/support/dict/type/page', query);
}

/** 查询字典类型详情 */
export function getDictType(id: string): Promise<DictTypeRes> {
  return get<DictTypeRes>(`/api/support/dict/type/${id}`);
}

/** 新增字典类型 */
export function createDictType(data: DictTypeSaveParams): Promise<null> {
  return post<null>('/api/support/dict/type', data);
}

/** 编辑字典类型 */
export function updateDictType(id: string, data: DictTypeSaveParams): Promise<null> {
  return put<null>(`/api/support/dict/type/${id}`, data);
}

/** 删除字典类型 */
export function deleteDictType(id: string): Promise<null> {
  return del<null>(`/api/support/dict/type/${id}`);
}

/** 分页查询字典项 */
export function pageDictItems(query: DictItemPageQuery): Promise<PageData<DictItemRes>> {
  return get<PageData<DictItemRes>>('/api/support/dict/item/page', query);
}

/** 查询字典项详情 */
export function getDictItem(id: string): Promise<DictItemRes> {
  return get<DictItemRes>(`/api/support/dict/item/${id}`);
}

/** 新增字典项 */
export function createDictItem(data: DictItemSaveParams): Promise<null> {
  return post<null>('/api/support/dict/item', data);
}

/** 编辑字典项 */
export function updateDictItem(id: string, data: DictItemSaveParams): Promise<null> {
  return put<null>(`/api/support/dict/item/${id}`, data);
}

/** 删除字典项 */
export function deleteDictItem(id: string): Promise<null> {
  return del<null>(`/api/support/dict/item/${id}`);
}

/** 按字典编码查询全部启用项，供下拉与标签渲染 */
export function listDictItems(dictCode: string): Promise<DictItemRes[]> {
  return get<DictItemRes[]>(`/api/support/dict/items/${dictCode}`);
}

/** 字典模块接口集合 */
export const dictApi = {
  pageDictTypes,
  getDictType,
  createDictType,
  updateDictType,
  deleteDictType,
  pageDictItems,
  getDictItem,
  createDictItem,
  updateDictItem,
  deleteDictItem,
  listDictItems,
};
