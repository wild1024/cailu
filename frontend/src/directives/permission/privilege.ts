import type { Directive, DirectiveBinding } from 'vue';
import { useUserStore } from '@/stores/user';

/// 校验当前用户是否拥有指令绑定的权限码之一。
///
/// # 参数
/// - `binding`：指令绑定对象，`value` 为权限码数组，如 `['system:user:add']`。
///
/// # 返回值
/// 命中任一权限码返回 `true`；未传权限码视为无需校验，返回 `true`。
function checkPrivilege(binding: DirectiveBinding): boolean {
  const value = binding.value;
  if (!Array.isArray(value) || value.length === 0) {
    return true;
  }
  const userStore = useUserStore();
  // 复用 store 的判定逻辑，统一处理超级管理员 `*:*:*` 通配
  return value.some((code) => userStore.hasPermission(code));
}

/// 根据权限判定结果切换元素的显示状态。
function applyPrivilege(el: HTMLElement, binding: DirectiveBinding): void {
  el.style.display = checkPrivilege(binding) ? '' : 'none';
}

/// v-privilege 指令：无权限时以 `display:none` 隐藏元素（保留 DOM 结构便于后续权限变化时恢复）。
export const privilege: Directive = {
  mounted(el: HTMLElement, binding: DirectiveBinding) {
    applyPrivilege(el, binding);
  },
  updated(el: HTMLElement, binding: DirectiveBinding) {
    applyPrivilege(el, binding);
  },
};
