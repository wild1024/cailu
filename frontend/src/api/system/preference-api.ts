/**
 * 用户界面偏好接口（P1 占位，后端尚未上线）
 *
 * 第一版配色方案只做 **localStorage 持久化**（键 `cailu-palette`），跨设备不同步。
 * 若后续要让"同一账号在任何机器上都用同一套配色"，需要后端补一张 `sys_user_preference`
 * 表 + 下面两个接口，然后把 `composables/usePalette.ts` 里的 `persist()` 补上远端写入、
 * `initPalette()` 补上远端读取（登录后以远端为准，未登录仍用本地）。
 *
 * ⚠ 当前状态：**未接入**。本文件只固化接口形状，避免将来各处各写一套命名。
 *   在接口上线前不要调用这里的函数（会命中不存在的路由并返回 404）。
 */
import { get, put } from '@/api/request';
import type { ColorMode, PaletteSelection } from '@/style/palettes';

/** 用户界面偏好 */
export interface UserPreference {
  /** 配色方案 id；`null` = 跟随默认（不应用任何方案） */
  palette: PaletteSelection;
  /** 明暗模式 */
  theme_mode: ColorMode;
  /** 侧边栏是否折叠 */
  side_compact: boolean;
}

/** 读取当前登录用户的界面偏好（P1） */
export function fetchPreference(): Promise<UserPreference> {
  return get<UserPreference>('/api/system/auth/preference');
}

/** 保存当前登录用户的界面偏好（P1） */
export function savePreference(data: Partial<UserPreference>): Promise<null> {
  return put<null>('/api/system/auth/preference', data);
}
