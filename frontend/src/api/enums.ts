import { get } from '@/api/request';
import type { TagTheme } from '@/types/theme';

/**
 * 枚举项（与后端 `shared::enums::EnumItem` 对应）。
 *
 * 与 `sys_dict` 的前端投影同构（`label` / `value` / `color`），因此字典标签与
 * 代码枚举可以共用同一套渲染逻辑；`value` 保留原生类型——整数枚举下发 number，
 * 编码枚举下发 string，回填表单时与接口契约一致，无需逐点转换。
 */
export type EnumItem = {
  /** 展示文案 */
  label: string;
  /** 取值（整数枚举为 number，编码枚举为 string） */
  value: string | number;
  /** 标签配色（TDesign `t-tag` 的 `theme`） */
  color: TagTheme;
  /** 补充说明（可选，供单选卡片的辅助文案） */
  description?: string;
};

/** 模块枚举表：枚举名 → 枚举项数组 */
export type ModuleEnums = Record<string, EnumItem[]>;

/**
 * 枚举模块 → 接口路径。
 *
 * 与后端 `#[alun::get(".../enums")]` 一一对应；新增枚举接口时在此登记一行。
 * 键名是前端唯一要背的约定（`{system|support|business}/{模块}`），页面组件通过
 * [`useModuleEnums`](../composables/useModuleEnums.ts) 按键取用。
 *
 * 只登记后端确有该路由的模块：无接口的键会让页面拿到 404 并降级，
 * 属于静默失败，不如不登记（未登记键在 `getModuleEnums` 直接抛错暴露）。
 */
const ENDPOINTS: Record<string, string> = {
  'support/config': '/api/support/config/enums',
};

/** 是否已登记该模块的枚举接口 */
export function hasEnumsEndpoint(moduleKey: string): boolean {
  return moduleKey in ENDPOINTS;
}

/**
 * 拉取一个模块的全部枚举。
 *
 * # 参数
 *
 * - `moduleKey`：模块键（`{system|support|business}/{模块}`，见 `ENDPOINTS`）。
 *
 * # 返回值
 *
 * 「枚举名 → 枚举项数组」；后端路由存在但模块无枚举时为空对象。
 *
 * # 异常情况
 *
 * - 未登记的模块键直接拒绝（编程错误，应在开发期暴露）；
 * - 401 / 403 / 5xx 由 `request.ts` 的统一拦截处理。
 */
export function getModuleEnums(moduleKey: string): Promise<ModuleEnums> {
  const url = ENDPOINTS[moduleKey];
  if (!url) {
    return Promise.reject(new Error(`未登记的枚举模块：${moduleKey}`));
  }
  return get<ModuleEnums>(url);
}
