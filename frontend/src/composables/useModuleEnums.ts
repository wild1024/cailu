import { onMounted } from 'vue';
import { useEnumsStore } from '@/stores/enums';

/**
 * 按模块取枚举的组件级入口。
 *
 * 把「模块键 + 懒加载 + 取值翻译」收拢到一个对象，页面里原来各写一份的
 * `xxxOptions` 常量表、`xxxLabel()` / `xxxTheme()` 函数直接换成这里的
 * `options()` / `label()` / `theme()`：
 *
 * ```ts
 * const enums = useModuleEnums('support/announcement');
 * // 模板里：<t-tag :theme="enums.theme('status', row.status)">{{ enums.label('status', row.status) }}</t-tag>
 * // 下拉里：:options="enums.options('status')"
 * ```
 *
 * # 参数
 *
 * - `moduleKey`：模块键（`{system|support|business}/{模块}`，与 `api/enums.ts` 登记一致）。
 *
 * # 返回值
 *
 * - `options(enumKey)`：下拉选项（`t-select` 的 `options` 直用）。
 * - `label(enumKey, value, fallback?)`：取值 → 文案，未命中回退原始值。
 * - `theme(enumKey, value)`：取值 → 标签配色，未命中回退 `default`。
 * - `description(enumKey, value)`：取值 → 补充说明，未命中为空串。
 * - `ready()`：等待本模块枚举拉取完成（供需要同步拿到选项的页面在 `onMounted` 里 `await`）。
 */
export function useModuleEnums(moduleKey: string) {
  const store = useEnumsStore();

  // 挂载即拉取：命中缓存时是无感的，失败静默（页面用回退文案，不阻断主流程）
  onMounted(() => {
    store.load(moduleKey).catch(() => undefined);
  });

  return {
    options: (enumKey: string) => store.itemsOf(moduleKey, enumKey).map((item) => ({ label: item.label, value: item.value })),
    label: (enumKey: string, value: string | number | null | undefined, fallback?: string) =>
      store.labelOf(moduleKey, enumKey, value, fallback),
    theme: (enumKey: string, value: string | number | null | undefined) => store.themeOf(moduleKey, enumKey, value),
    description: (enumKey: string, value: string | number | null | undefined) =>
      store.descriptionOf(moduleKey, enumKey, value),
    ready: () => store.load(moduleKey),
  };
}
