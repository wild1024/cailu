import { ref } from 'vue';
import { defineStore } from 'pinia';
import { getModuleEnums, type EnumItem, type ModuleEnums } from '@/api/enums';
import type { TagTheme } from '@/types/theme';

/**
 * 模块枚举缓存：按模块拉取一次、多页共用。
 *
 * 单一事实源在后端（`GET /{system|support}/{module}/enums`），前端只做「取一次 →
 * 缓存 → 渲染」。同一模块内多个页面（列表筛选、表单下拉、标签配色）共享一份
 * 缓存，会话期间不重复请求；登出时由 `auth` 登出逻辑统一 `clear()`。
 */
export const useEnumsStore = defineStore('enums', () => {
  /** 模块键 → 枚举表（仅成功拉取过的模块有值） */
  const tables = ref<Record<string, ModuleEnums>>({});

  /** 拉取失败的模块键 → true（失败不缓存数据，但避免同一会话内反复重试刷屏） */
  const broken = ref<Record<string, boolean>>({});

  /**
   * 读取模块枚举，命中缓存时直接返回。
   *
   * # 参数
   *
   * - `moduleKey`：模块键。
   *
   * # 返回值
   *
   * 枚举表；首次调用时发起请求，失败时返回空表并标记 `broken`（页面降级为
   * 本地回退文案，不阻断主流程）。
   */
  async function load(moduleKey: string): Promise<ModuleEnums> {
    const cached = tables.value[moduleKey];
    if (cached) {
      return cached;
    }
    if (broken.value[moduleKey]) {
      return {};
    }
    try {
      const table = await getModuleEnums(moduleKey);
      tables.value[moduleKey] = table;
      return table;
    } catch (error) {
      broken.value[moduleKey] = true;
      throw error;
    }
  }

  /** 取枚举项数组；未加载或无该枚举时为空数组 */
  function itemsOf(moduleKey: string, enumKey: string): EnumItem[] {
    return tables.value[moduleKey]?.[enumKey] ?? [];
  }

  /**
   * 取值 → 展示文案。
   *
   * # 参数
   *
   * - `moduleKey` / `enumKey`：定位枚举表。
   * - `value`：任意取值（number / string 均可，内部统一按 `String` 比较）。
   * - `fallback`：未命中时的回退文案，缺省回显原始值。
   */
  function labelOf(
    moduleKey: string,
    enumKey: string,
    value: string | number | null | undefined,
    fallback?: string,
  ): string {
    if (value === null || value === undefined || value === '') {
      return fallback ?? '';
    }
    const hit = itemsOf(moduleKey, enumKey).find((item) => String(item.value) === String(value));
    return hit?.label ?? fallback ?? String(value);
  }

  /** 取值 → 标签配色；未命中回退 `default` */
  function themeOf(
    moduleKey: string,
    enumKey: string,
    value: string | number | null | undefined,
  ): TagTheme {
    const hit = itemsOf(moduleKey, enumKey).find((item) => String(item.value) === String(value));
    return (hit?.color ?? 'default') as TagTheme;
  }

  /** 是否存在「取值 → description」的补充说明 */
  function descriptionOf(
    moduleKey: string,
    enumKey: string,
    value: string | number | null | undefined,
  ): string {
    const hit = itemsOf(moduleKey, enumKey).find((item) => String(item.value) === String(value));
    return hit?.description ?? '';
  }

  /** 清空缓存；传入模块键时仅清理该模块（便于登出与调试） */
  function clear(moduleKey?: string): void {
    if (moduleKey) {
      delete tables.value[moduleKey];
      delete broken.value[moduleKey];
      return;
    }
    tables.value = {};
    broken.value = {};
  }

  return { tables, broken, load, itemsOf, labelOf, themeOf, descriptionOf, clear };
});
