import { ref } from 'vue';
import { defineStore } from 'pinia';

/** 主题模式 */
type ThemeMode = 'light' | 'dark';

/** 持久化键：与后端无关的纯前端偏好 */
const SETTING_KEY = 'cailu-setting';

/** 从 localStorage 恢复上次保存的偏好（损坏时静默回退默认值） */
function loadSettings(): { themeMode: ThemeMode; sideCompact: boolean } {
  try {
    const saved = localStorage.getItem(SETTING_KEY);
    if (saved) {
      const parsed = JSON.parse(saved) as Partial<{ themeMode: ThemeMode; sideCompact: boolean }>;
      return {
        themeMode: parsed.themeMode === 'dark' ? 'dark' : 'light',
        sideCompact: parsed.sideCompact === true,
      };
    }
  } catch {
    // 解析失败按默认值处理
  }
  return { themeMode: 'light', sideCompact: false };
}

/** 界面偏好：主题模式与侧边栏折叠状态（两个布局共用） */
export const useSettingStore = defineStore('setting', () => {
  const defaults = loadSettings();

  /** 主题模式：light / dark（TDesign 通过 html[theme-mode] 生效） */
  const themeMode = ref<ThemeMode>(defaults.themeMode);
  /** 侧边栏折叠状态（管理布局） */
  const sideCompact = ref(defaults.sideCompact);

  /** 把当前主题写到 html 根元素上，全局生效 */
  function applyTheme(): void {
    document.documentElement.setAttribute('theme-mode', themeMode.value);
  }

  /** 切换明暗主题并持久化 */
  function toggleTheme(): void {
    themeMode.value = themeMode.value === 'dark' ? 'light' : 'dark';
    applyTheme();
    localStorage.setItem(SETTING_KEY, JSON.stringify({ themeMode: themeMode.value, sideCompact: sideCompact.value }));
  }

  /** 切换侧边栏折叠并持久化 */
  function toggleCompact(): void {
    sideCompact.value = !sideCompact.value;
    localStorage.setItem(SETTING_KEY, JSON.stringify({ themeMode: themeMode.value, sideCompact: sideCompact.value }));
  }

  /** 应用启动时调用一次，恢复上次保存的主题 */
  function initTheme(): void {
    applyTheme();
  }

  return { themeMode, sideCompact, toggleTheme, toggleCompact, initTheme };
});
