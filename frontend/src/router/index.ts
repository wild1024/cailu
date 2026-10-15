import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { useUserStore } from '@/stores/user';

/** 路由元信息：`title` 供面包屑与页面标题使用，`public` 标记免登录页面 */
declare module 'vue-router' {
  interface RouteMeta {
    /** 页面标题 */
    title?: string;
    /** 是否为免登录页面 */
    public?: boolean;
    /** 访问该页面所需的权限码 */
    permission?: string;
  }
}

/** 业务路由表：权限码与后端菜单种子数据保持一致 */
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/dashboard',
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: '登录', public: true },
  },
  {
    path: '/',
    component: () => import('@/layouts/BasicLayout.vue'),
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/dashboard/index.vue'),
        meta: { title: '首页' },
      },
      {
        path: 'system/user',
        name: 'SystemUser',
        component: () => import('@/views/system/user/index.vue'),
        meta: { title: '用户管理', permission: 'system:user:list' },
      },
      {
        path: 'system/role',
        name: 'SystemRole',
        component: () => import('@/views/system/role/index.vue'),
        meta: { title: '角色管理', permission: 'system:role:list' },
      },
      {
        path: 'system/menu',
        name: 'SystemMenu',
        component: () => import('@/views/system/menu/index.vue'),
        meta: { title: '菜单管理', permission: 'system:menu:list' },
      },
      {
        path: 'system/dept',
        name: 'SystemDept',
        component: () => import('@/views/system/dept/index.vue'),
        meta: { title: '部门管理', permission: 'system:dept:list' },
      },
      {
        path: 'support/dict',
        name: 'SupportDict',
        component: () => import('@/views/support/dict/index.vue'),
        meta: { title: '字典管理', permission: 'support:dict:list' },
      },
      {
        path: 'support/config',
        name: 'SupportConfig',
        component: () => import('@/views/support/config/index.vue'),
        meta: { title: '参数配置', permission: 'support:config:list' },
      },
      {
        path: 'support/file',
        name: 'SupportFile',
        component: () => import('@/views/support/file/index.vue'),
        meta: { title: '文件管理', permission: 'support:file:list' },
      },
      {
        path: 'support/log',
        name: 'SupportLog',
        component: () => import('@/views/support/log/index.vue'),
        meta: { title: '操作日志', permission: 'support:log:list' },
      },
      {
        path: 'support/job',
        name: 'SupportJob',
        component: () => import('@/views/support/job/index.vue'),
        meta: { title: '定时任务', permission: 'support:job:list' },
      },
      {
        path: 'support/online',
        name: 'SupportOnline',
        component: () => import('@/views/support/online/index.vue'),
        meta: { title: '在线用户', permission: 'support:online:list' },
      },
      {
        path: 'user/profile',
        name: 'UserProfile',
        component: () => import('@/views/user/profile.vue'),
        meta: { title: '个人中心' },
      },
      {
        path: 'result/403',
        name: 'Result403',
        component: () => import('@/views/result/403/index.vue'),
        meta: { title: '无访问权限' },
      },
      {
        path: 'result/404',
        name: 'Result404',
        component: () => import('@/views/result/404/index.vue'),
        meta: { title: '页面不存在' },
      },
      {
        path: 'result/500',
        name: 'Result500',
        component: () => import('@/views/result/500/index.vue'),
        meta: { title: '服务异常' },
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/result/404/index.vue'),
    meta: { title: '页面不存在', public: true },
  },
];

export const router = createRouter({
  history: createWebHistory(import.meta.env.VITE_APP_CONTEXT_PATH || '/'),
  routes,
});

// 全局前置守卫：未登录跳登录页；已登录访问登录页跳首页；并校验路由权限码
router.beforeEach(async (to) => {
  const userStore = useUserStore();
  const isPublic = to.meta.public === true;

  if (!userStore.token) {
    if (isPublic) {
      return true;
    }
    return { path: '/login', query: { redirect: to.fullPath } };
  }

  // 刷新页面后 Pinia 状态为空，需先补齐用户信息与菜单
  if (!userStore.loaded) {
    try {
      await userStore.loadProfile();
    } catch {
      userStore.reset();
      return { path: '/login', query: { redirect: to.fullPath } };
    }
  }

  if (to.path === '/login') {
    return { path: '/dashboard' };
  }

  const permission = to.meta.permission;
  if (permission && !userStore.hasPermission(permission)) {
    return { path: '/result/403' };
  }

  return true;
});

// 同步页面标题，便于多标签页区分
router.afterEach((to) => {
  const title = to.meta.title;
  document.title = typeof title === 'string' ? `${title} - 菜芦管理后台` : '菜芦管理后台';
});

export default router;