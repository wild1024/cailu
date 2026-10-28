<template>
  <t-layout class="basic-layout">
    <t-aside class="basic-layout__aside" :width="asideWidth">
      <div class="basic-layout__brand" role="button" tabindex="0" @click="goHome" @keydown.enter="goHome">
        <span class="basic-layout__logo"><img src="/logo.png" alt="菜芦" /></span>
        <span v-show="!collapsed" class="basic-layout__brand-name">菜芦后台管理</span>
      </div>

      <t-menu
        class="basic-layout__menu"
        :value="activeMenu"
        :collapsed="collapsed"
        :collapsed-width="64"
        :expand-mutex="true"
        :theme="setting.themeMode === 'dark' ? 'dark' : 'light'"
        @change="onMenuChange"
      >
        <side-menu-item v-for="item in userStore.navMenus" :key="item.id" :item="item" />
      </t-menu>
    </t-aside>

    <t-layout class="basic-layout__main">
      <t-header class="basic-layout__header">
        <t-button
          class="basic-layout__collapse"
          variant="text"
          shape="square"
          @click="collapsed = !collapsed"
        >
          <t-icon :name="collapsed ? 'menu-unfold' : 'menu-fold'" />
        </t-button>
        <t-breadcrumb class="basic-layout__breadcrumb">
          <t-breadcrumb-item
            v-for="(crumb, index) in breadcrumbs"
            :key="`${crumb.title}-${index}`"
            :class="{ 'is-link': crumb.to }"
            @click="crumb.to ? goTo(crumb.to) : undefined"
          >
            {{ crumb.title }}
          </t-breadcrumb-item>
        </t-breadcrumb>

        <!-- 配色方案快捷切换（用户不选则完全不影响界面，走 theme.less 原有配色） -->
        <palette-switch />

        <t-tooltip :content="setting.themeMode === 'dark' ? '切换亮色主题' : '切换暗色主题'">
          <t-button
            class="basic-layout__theme-toggle"
            variant="text"
            shape="square"
            @click="setting.toggleTheme"
          >
            <t-icon :name="setting.themeMode === 'dark' ? 'sunny' : 'moon'" />
          </t-button>
        </t-tooltip>

        <t-dropdown :options="userActions" :popup-props="userPopupProps" @click="onUserAction">
          <button class="basic-layout__user" type="button">
            <t-avatar  size="medium">{{ avatarText }}</t-avatar>
            <span class="basic-layout__username">{{ userStore.displayName || '未登录' }}</span>
            <t-icon class="basic-layout__caret" name="chevron-down" />
          </button>
        </t-dropdown>
      </t-header>

      <t-content class="basic-layout__content">
        <router-view v-slot="{ Component }">
          <transition name="page-fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </t-content>
    </t-layout>
  </t-layout>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { DialogPlugin, type DropdownOption } from 'tdesign-vue-next';
import SideMenuItem from '@/layouts/components/SideMenuItem.vue';
import PaletteSwitch from '@/layouts/components/PaletteSwitch.vue';
import { useUserStore } from '@/stores/user';
import { useSettingStore } from '@/stores/setting';
import { normalizeMenuPath, collectLeafPaths } from '@/utils/menu';

/** 用户下拉菜单项 */
const userActions: DropdownOption[] = [
  { content: '个人中心', value: 'profile' },
  { content: '外观设置', value: 'preference' },
  { content: '退出登录', value: 'logout' },
];

/** 用户下拉面板配置：追加类名便于放大面板与选项点击区域（弹层挂载到 body，样式见文末非 scoped 块） */
const userPopupProps = {
  overlayClassName: 'basic-layout__user-dropdown',
};

/** 面包屑条目：`to` 非空时该条可点击跳转 */
interface BreadcrumbItem {
  /** 显示文本 */
  title: string;
  /** 目标路由路径；为空表示纯文本（不可点击） */
  to?: string;
}

const route = useRoute();
const router = useRouter();
const userStore = useUserStore();
const setting = useSettingStore();

/** 应用启动时恢复上次保存的主题（明暗），避免刷新后回到默认亮色 */
setting.initTheme();

/** 侧边栏折叠状态 */
const collapsed = ref(false);

/** 侧边栏宽度：折叠后仅保留图标 */
const asideWidth = computed(() => (collapsed.value ? '64px' : '232px'));

/** 头像占位文字：取展示名首字符 */
const avatarText = computed(() => (userStore.displayName || 'U').slice(0, 1).toUpperCase());

/** 可跳转的叶子菜单路径，避免点击目录时跳到不存在的路由 */
const leafPaths = computed(() => collectLeafPaths(userStore.navMenus));

/** 当前高亮的菜单项 */
const activeMenu = computed(() => {
  const path = normalizeMenuPath(route.path);
  return leafPaths.value.has(path) ? path : '';
});

/** 面包屑：首页（可点击回首页）+ 当前匹配到的带标题路由（末级为纯文本） */
const breadcrumbs = computed<BreadcrumbItem[]>(() => {
  const titles = route.matched
    .map((record) => record.meta.title)
    .filter((title): title is string => typeof title === 'string');
  if (route.path === '/dashboard') {
    return titles.map((title) => ({ title }));
  }
  return [{ title: '首页', to: '/dashboard' }, ...titles.map((title) => ({ title }))];
});

/** 跳转首页 */
function goHome(): void {
  if (route.path !== '/dashboard') {
    void router.push('/dashboard');
  }
}

/** 面包屑条目跳转（仅可点击条目传入路径） */
function goTo(path: string): void {
  if (route.path !== path) {
    void router.push(path);
  }
}

/** 仅叶子菜单参与路由跳转 */
function onMenuChange(value: unknown): void {
  const path = typeof value === 'string' ? value : '';
  if (path && leafPaths.value.has(path)) {
    void router.push(path);
  }
}

function onUserAction(item: DropdownOption): void {
  if (item.value === 'profile') {
    void router.push('/user/profile');
    return;
  }
  if (item.value === 'preference') {
    void router.push('/user/preference');
    return;
  }
  if (item.value === 'logout') {
    openLogoutConfirm();
  }
}

/** 退出登录二次确认 */
function openLogoutConfirm(): void {
  const dialog = DialogPlugin.confirm({
    header: '退出登录',
    body: '确认退出当前账号？',
    confirmBtn: { content: '退出', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void doLogout();
    },
  });
}

async function doLogout(): Promise<void> {
  await userStore.logout();
  await router.push('/login');
}
</script>

<style scoped lang="less">
.basic-layout {
  height: 100%;

  &__aside {
    display: flex;
    flex-direction: column;
    // 浅色侧边栏：与亮色菜单统一配色，右描边与主区分隔
    background-color: var(--td-bg-color-container);
    border-right: 1px solid var(--td-component-stroke);
    // TDesign 的 .t-layout__sider 默认 flex-shrink:1，会被内容超宽的主区挤到 0 宽，
    // 这里固定不收缩，配合 .basic-layout__main 的 min-width:0 让主区自行收缩滚动
    flex: none;
    overflow: hidden;
    transition: width 0.2s ease;
  }

  &__brand {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 56px;
    padding: 0 16px;
    white-space: nowrap;
    overflow: hidden;
    cursor: pointer;
    border-bottom: 1px solid var(--td-component-stroke);
    transition: background-color 0.2s;

    &:hover {
      background-color: var(--td-bg-color-container-hover);
    }
  }

  &__logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 28px;
    height: 28px;

    img {
      width: 100%;
      height: 100%;
      object-fit: contain;
      border-radius: 8px;
    }
  }

  &__brand-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--td-text-color-primary);
    letter-spacing: 0.5px;
  }

  &__menu {
    flex: 1;
    overflow-x: hidden;
    overflow-y: auto;
    // 让菜单透明，透出侧边栏底色，避免出现色块断层
    background: transparent;
    border-right: none;
  }

  &__main {
    min-width: 0;
  }

  &__header {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 56px;
    padding: 0 16px 0 8px;
    background-color: var(--td-bg-color-container);
    border-bottom: 1px solid var(--td-component-stroke);
    box-shadow: 0 1px 4px rgb(0 0 0 / 4%);
  }

  &__collapse {
    flex-shrink: 0;
    color: var(--td-text-color-secondary);
  }

  // 主题切换按钮：与折叠按钮同款观感，置于用户区左侧
  &__theme-toggle {
    flex-shrink: 0;
    color: var(--td-text-color-secondary);
  }

  &__breadcrumb {
    flex: 1;
    min-width: 0;

    // 可点击的面包屑条目：手型光标 + 悬浮高亮
    :deep(.is-link) {
      cursor: pointer;
      transition: color 0.2s;

      &:hover {
        color: var(--td-brand-color);
      }
    }
  }

  &__user {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    // 放大点击热区，避免右上角个人中心入口过小难点
    height: 44px;
    padding: 0 14px;
    cursor: pointer;
    background: transparent;
    border: none;
    border-radius: 10px;
    transition: background-color 0.2s;

    &:hover {
      background-color: var(--td-bg-color-container-hover);
    }
  }

  &__avatar {
    background: linear-gradient(135deg, #4b8bff 0%, #7b5cff 100%);
  }

  &__username {
    font-size: 15px;
    color: var(--td-text-color-primary);
  }

  &__caret {
    font-size: 16px;
    color: var(--td-text-color-placeholder);
  }

  &__content {
    padding: 16px;
    overflow: auto;
    background-color: var(--td-bg-color-page);
  }
}

// 路由切换过渡：淡入 + 轻微上移，避免页面生硬闪现
.page-fade-enter-active,
.page-fade-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.page-fade-leave-to {
  opacity: 0;
}
</style>

<!-- 用户下拉面板挂载到 body，需用非 scoped 样式才能命中：整体加宽、选项加大一号 -->
<style lang="less">
.basic-layout__user-dropdown {
  .t-dropdown__menu {
    min-width: 172px;
    padding: 8px;
  }

  .t-dropdown__item {
    // 覆盖组件内联的 max-width/min-width（默认 100px/10px），让选项撑满面板宽度
    width: 100% !important;
    max-width: none !important;
    min-width: 0 !important;
    box-sizing: border-box;
    min-height: 40px;
    padding: 0 16px;
    font-size: 15px;
    line-height: 40px;
  }
}
</style>
