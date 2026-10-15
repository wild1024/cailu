import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import {
  fetchMe,
  fetchMenus,
  login as loginApi,
  logout as logoutApi,
  type CurrentUserRes,
  type LoginParams,
  type MenuNode,
} from '@/api/system/auth-api';
import { clearTokens, REFRESH_TOKEN_KEY, saveTokens, TOKEN_KEY } from '@/api/request';

/** 超级管理员通配权限码 */
const SUPER_PERMISSION = '*:*:*';

/** 参与导航渲染的菜单类型：目录与菜单（按钮类型仅作为权限点） */
const NAV_MENU_TYPES: number[] = [1, 2];

/** 递归过滤菜单树：剔除隐藏节点、按钮节点以及当前用户无权限的节点 */
function filterNavMenus(nodes: MenuNode[], hasPermission: (code: string) => boolean): MenuNode[] {
  const result: MenuNode[] = [];
  for (const node of nodes) {
    if (!NAV_MENU_TYPES.includes(node.menu_type) || !node.is_visible) {
      continue;
    }
    if (!hasPermission(node.auth_code)) {
      continue;
    }
    const children = filterNavMenus(node.children ?? [], hasPermission);
    // 目录下若无可见子菜单则整体隐藏，避免出现空目录
    if (node.menu_type === 1 && children.length === 0) {
      continue;
    }
    result.push({ ...node, children });
  }
  return result;
}

/** 用户会话状态：令牌、当前用户、权限与导航菜单 */
export const useUserStore = defineStore('user', () => {
  /** 访问令牌 */
  const token = ref(localStorage.getItem(TOKEN_KEY) ?? '');
  /** 刷新令牌 */
  const refreshToken = ref(localStorage.getItem(REFRESH_TOKEN_KEY) ?? '');
  /** 当前登录用户信息 */
  const userInfo = ref<CurrentUserRes | null>(null);
  /** 后端下发的菜单树 */
  const menus = ref<MenuNode[]>([]);
  /** 是否已完成用户信息加载（路由守卫据此决定是否拉取） */
  const loaded = ref(false);

  /** 权限码列表 */
  const permissions = computed<string[]>(() => userInfo.value?.permissions ?? []);
  /** 角色编码列表 */
  const roles = computed<string[]>(() => userInfo.value?.roles ?? []);
  /** 顶栏展示名称，优先真实姓名 */
  const displayName = computed<string>(() => userInfo.value?.real_name || userInfo.value?.username || '');
  /** 侧边栏菜单树（已按可见性与权限过滤） */
  const navMenus = computed<MenuNode[]>(() => filterNavMenus(menus.value, hasPermission));

  /**
   * 判断是否拥有某个权限码。
   *
   * 未传 code 视为无需校验；持有 `*:*:*` 的超级管理员恒为 true。
   */
  function hasPermission(code?: string | null): boolean {
    if (!code) {
      return true;
    }
    const list = permissions.value;
    return list.includes(SUPER_PERMISSION) || list.includes(code);
  }

  /** 登录并加载用户信息与菜单 */
  async function login(params: LoginParams): Promise<void> {
    const res = await loginApi(params);
    token.value = res.access_token;
    refreshToken.value = res.refresh_token;
    // 令牌必须同步落盘：后续请求的 Authorization 头由 axios 拦截器从 localStorage 注入
    saveTokens(res.access_token, res.refresh_token);
    await loadProfile();
  }

  /** 拉取当前用户信息与菜单树，供路由守卫与布局使用 */
  async function loadProfile(): Promise<void> {
    const [me, menuTree] = await Promise.all([fetchMe(), fetchMenus()]);
    userInfo.value = me;
    menus.value = menuTree;
    loaded.value = true;
  }

  /** 清空本地会话状态 */
  function reset(): void {
    clearTokens();
    token.value = '';
    refreshToken.value = '';
    userInfo.value = null;
    menus.value = [];
    loaded.value = false;
  }

  /** 登出：无论后端是否成功，本地会话必须清理 */
  async function logout(): Promise<void> {
    try {
      await logoutApi();
    } catch {
      // 后端登出失败（如令牌已过期）不影响前端清理，忽略即可
    }
    reset();
  }

  return {
    token,
    refreshToken,
    userInfo,
    menus,
    loaded,
    permissions,
    roles,
    displayName,
    navMenus,
    hasPermission,
    login,
    loadProfile,
    logout,
    reset,
  };
});