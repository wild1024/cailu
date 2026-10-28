/**
 * 配色方案的应用 / 切换逻辑
 *
 * 职责边界：
 * - `style/palettes.ts` 只做纯计算（色板 → 变量表 + 对比度检查），不碰 DOM、不碰 Vue；
 * - 本文件负责「把变量表落地到 `<html>` 内联 style」+「持久化」+「与明暗模式联动」。
 *
 * ## 核心语义：**默认不应用任何方案**（自主选择）
 *
 * - 用户**没有**选择 → 不向 `<html>` 注入任何 CSS 变量、不写任何 `data-*` 属性，
 *   `theme.less` 里的原有配色原样生效 —— 升级对老用户零感知；
 * - 用户**主动选择**了某套方案 → 才注入该套方案的变量；
 * - 「跟随默认 / 恢复默认」→ 清掉 localStorage 与 `<html>` 上的全部注入，回到原样。
 *
 * 为什么写内联 style 而不是注入 `<style>`：
 *   TDesign 把品牌色写在 `:root[theme-mode='light'|'dark']` 里（特异性 0,1,1），
 *   `theme.less` 靠"同特异性 + 后加载"取胜。注入的 `<style>` 同样是 0,1,1，
 *   一旦有第三方样式晚于我们加载就会被压回去。内联 style 在层叠中优先级最高，
 *   任何选择器都盖不掉，所以换肤结果稳定、且与加载顺序无关。
 *
 * 与明暗模式的关系（8 套 × 明暗 2 = 16 组）：
 *   明暗模式仍由 `stores/setting.ts` 的 `themeMode` 单一控制（顶栏按钮、设置页、快捷面板
 *   都走同一个 setter），本文件只订阅它并在变化时**重算整套变量**。
 *   因此切换配色不会丢明暗状态，切换明暗也不会丢配色 —— 两者是正交的两个维度。
 *   未选择方案时明暗行为与升级前完全一致（只切 `theme-mode`，不注入任何变量）。
 */
import { computed, readonly, ref, watch } from 'vue';
import { useSettingStore } from '@/stores/setting';
import {
  DEFAULT_PALETTE_ID,
  PALETTES,
  buildAllPalettes,
  buildPalette,
  findPalette,
  isPaletteId,
  type ColorMode,
  type PaletteBuild,
  type PaletteDef,
  type PaletteSelection,
} from '@/style/palettes';

/** 持久化键（与现有 `cailu-setting` 命名保持一致） */
export const PALETTE_STORAGE_KEY = 'cailu-palette';

/**
 * 全部方案可能写入的变量名（取并集）。
 *
 * 「跟随默认」时需要一个**确定的**变量名集合来逐条 `removeProperty`：
 * 目前各方案推导出的变量名相同，取并集可防止将来某套方案新增变量后清不干净。
 */
const ALL_PALETTE_VARS: string[] = Array.from(
  new Set(buildAllPalettes().flatMap((build) => Object.keys(build.vars))),
);

/** localStorage 里存的结构 */
interface StoredPalette {
  /** 方案 id（「跟随默认」时不落盘，读到即视为未选择） */
  palette: PaletteSelection;
  /** 明暗模式（仅作镜像记录，运行时仍以 setting store 为准） */
  mode: ColorMode;
  /** 最近更新时间（便于排查"用户到底选了啥"） */
  updatedAt: string;
}

/**
 * 读取已保存的方案。
 *
 * 无值 / `null` / 非法 id / 脏 JSON 一律回退「跟随默认」（`null`）——
 * 绝不能回退成某个具体方案，否则"没选过"的用户也会被套上配色。
 */
function readStoredPalette(): PaletteSelection {
  try {
    const raw = localStorage.getItem(PALETTE_STORAGE_KEY);
    if (!raw) return DEFAULT_PALETTE_ID;
    const parsed: unknown = JSON.parse(raw);
    // 兼容早期可能直接存字符串的形式
    if (typeof parsed === 'string') {
      return isPaletteId(parsed) ? parsed : DEFAULT_PALETTE_ID;
    }
    const candidate = (parsed as Partial<StoredPalette> | null)?.palette;
    return isPaletteId(candidate) ? candidate : DEFAULT_PALETTE_ID;
  } catch {
    return DEFAULT_PALETTE_ID;
  }
}

/** 当前选中的方案（模块级单例：全应用共享同一份状态）；`null` = 跟随默认 */
const paletteId = ref<PaletteSelection>(readStoredPalette());

/** 最近一次构建结果（设置页的对比度表格直接读它）；「跟随默认」时为 `null` */
const lastBuild = ref<PaletteBuild | null>(
  paletteId.value ? buildPalette(paletteId.value, 'light') : null,
);

/** 变量版本号：每次真正变更加入 CSS 变量自增，ECharts 等需要命令式刷新的地方据此重绘 */
const paletteVersion = ref(0);

/** 已应用到 DOM 的组合（避免同一次切换重复写上百条变量） */
let appliedKey = '';

/** 明暗模式订阅是否已建立（避免多次 usePalette 重复 watch） */
let watching = false;

/** 取 setting store（保证在 pinia 安装之后调用） */
function setting() {
  return useSettingStore();
}

/** 当前明暗模式：唯一来源是 setting store */
function currentMode(): ColorMode {
  return setting().themeMode;
}

/**
 * 清掉 `<html>` 上的全部配色注入。
 *
 * 除了逐条 `removeProperty`，还必须清掉两个状态属性：`palette-bridge.less` 里有
 * `:root[data-palette-mode='dark'] ...` 规则，属性残留会让深色态的特殊规则继续生效。
 */
function clearInjectedVars(): void {
  const root = document.documentElement;
  for (const name of ALL_PALETTE_VARS) {
    root.style.removeProperty(name);
  }
  root.removeAttribute('data-palette');
  root.removeAttribute('data-palette-mode');
}

/**
 * 把当前「方案 × 模式」写成 `<html>` 上的 CSS 变量。
 *
 * 当前选择为 `null`（跟随默认）时：清掉所有注入，**什么都不写** —— 这是「界面与升级前一致」的落点。
 *
 * @param force 为 true 时即便组合未变也重写（设置页"重新应用"用）
 * @returns 本次构建结果；「跟随默认」时返回 `null`
 */
export function applyPalette(force = false): PaletteBuild | null {
  const mode = currentMode();
  const id = paletteId.value;
  const key = `${id ?? 'none'}:${mode}`;

  if (!force && key === appliedKey) {
    return lastBuild.value;
  }

  if (id === null) {
    // 跟随默认：不注入任何变量，theme.less 原样生效
    clearInjectedVars();
    lastBuild.value = null;
    appliedKey = key;
    paletteVersion.value += 1;
    return null;
  }

  const build = buildPalette(id, mode);
  // 先清一遍：避免上一套方案里本方案没有的变量残留（目前各方案变量名一致，留作保险）
  clearInjectedVars();
  for (const [name, value] of Object.entries(build.vars)) {
    document.documentElement.style.setProperty(name, value);
  }
  // 附带两个状态属性：CSS 可据此做特例，自动化测试也靠它断言当前组合
  document.documentElement.setAttribute('data-palette', id);
  document.documentElement.setAttribute('data-palette-mode', mode);
  lastBuild.value = build;
  appliedKey = key;
  paletteVersion.value += 1;

  // 自动修正/色值推断要在控制台留痕，避免"静默输出不合规配色"
  if (build.warnings.length) {
    console.warn(`[palette] ${build.palette.name}（${mode}）已自动修正：`, build.warnings.join('；'));
  }
  return build;
}

/**
 * 写入 / 清除 localStorage。
 *
 * 「跟随默认」= 不落盘（直接删键），这样「没选过」与「选了又恢复默认」在存储上完全等价。
 * 隐私模式 / 配额满导致写入失败时不影响主流程。
 */
function persist(): void {
  try {
    if (paletteId.value === null) {
      localStorage.removeItem(PALETTE_STORAGE_KEY);
      return;
    }
    const payload: StoredPalette = {
      palette: paletteId.value,
      mode: currentMode(),
      updatedAt: new Date().toISOString(),
    };
    localStorage.setItem(PALETTE_STORAGE_KEY, JSON.stringify(payload));
  } catch {
    // 隐私模式 / 配额满：忽略，本次会话内仍然生效
  }
  // P1（可选）：登录用户同步到后端用户偏好，见 `src/api/system/preference-api.ts`，
  // 后端接口未上线前保持空实现，不阻塞第一版本地持久化。
}

/**
 * 切换方案：立即生效（无刷新、无需保存按钮）并持久化。
 *
 * @param id 8 套方案之一的 id；传 `null` 表示「跟随默认（保持原主题）」
 */
export function setPalette(id: PaletteSelection): void {
  const next = isPaletteId(id) ? id : null;
  if (next === paletteId.value) {
    applyPalette(true);
    return;
  }
  paletteId.value = next;
  applyPalette();
  persist();
}

/** 切换明暗模式：交给 setting store（它负责写 `theme-mode` 属性与 `cailu-setting`） */
export function setPaletteMode(mode: ColorMode): void {
  const store = setting();
  if (store.themeMode !== mode) {
    store.setThemeMode(mode);
  }
  // store 的 watch 会触发重算；这里显式再调一次，覆盖"模式未变但要刷新"的场景
  applyPalette(true);
  persist();
}

/** 回到默认：清 localStorage + 移除 `<html>` 上的全部内联变量，原主题立刻生效 */
export function resetPalette(): void {
  paletteId.value = DEFAULT_PALETTE_ID;
  applyPalette(true);
  persist();
}

/**
 * 应用启动时调用一次（main.ts 在 mount 之前）。
 *
 * 顺序很重要：先让 setting store 把 `theme-mode` 写到 `<html>`，再应用配色变量，
 * 这样首屏渲染直接用最终配色，不会先闪一帧默认色再跳色。
 * 未选择方案时本函数只建立一个 watch，不改动 DOM 上任何颜色。
 */
export function initPalette(): void {
  if (!watching) {
    // 明暗模式变化 → 整套变量要重算（深色模式的底色/文字/品牌色都是另一套推导结果）
    watch(() => setting().themeMode, () => applyPalette(), { flush: 'sync' });
    watching = true;
  }
  applyPalette(true);
}

/**
 * 组合式入口。
 *
 * 在组件里用：
 * ```ts
 * const { paletteId, palettes, currentPalette, isDefault, setPalette, mode, setPaletteMode, build } = usePalette();
 * ```
 */
export function usePalette() {
  if (!watching) {
    // 组件首次调用时兜底建立订阅（正常情况下 main.ts 已经初始化过）
    initPalette();
  }

  /** 当前方案定义；「跟随默认」时为 `null` */
  const currentPalette = computed<PaletteDef | null>(() => findPalette(paletteId.value));

  /** 是否处于「跟随默认」（未应用任何方案） */
  const isDefault = computed(() => paletteId.value === null);

  /** 当前「方案 × 模式」的完整构建结果；「跟随默认」时为 `null` */
  const build = computed<PaletteBuild | null>(() =>
    paletteId.value ? buildPalette(paletteId.value, currentMode()) : null,
  );

  /** 各方案在当前模式下的构建结果（设置页色卡上的"是否需压暗版"标记用） */
  const buildsByPalette = computed<Record<string, PaletteBuild>>(() => {
    const map: Record<string, PaletteBuild> = {};
    for (const palette of PALETTES) {
      map[palette.id] = buildPalette(palette.id, currentMode());
    }
    return map;
  });

  return {
    /** 全部方案定义 */
    palettes: PALETTES,
    /** 当前方案 id（只读，切换请用 setPalette，避免绕过持久化）；`null` = 跟随默认 */
    paletteId: readonly(paletteId),
    /** 当前方案定义；`null` = 跟随默认 */
    currentPalette,
    /** 是否「跟随默认」 */
    isDefault,
    /** 当前明暗模式 */
    mode: computed<ColorMode>(() => currentMode()),
    /** 当前组合的构建结果；跟随默认时 `null` */
    build,
    /** 所有方案（当前模式）的构建结果 */
    buildsByPalette,
    /** 变量版本号（图表重绘依赖） */
    version: readonly(paletteVersion),
    /** 最近一次应用的构建结果 */
    applied: readonly(lastBuild),
    /** 切换方案（传 `null` = 跟随默认） */
    setPalette,
    /** 切换明暗 */
    setMode: setPaletteMode,
    /** 切回「跟随默认（保持原主题）」 */
    resetPalette,
    /** 强制重算并重写变量 */
    reapply: () => applyPalette(true),
  };
}

/**
 * 图表配色（ECharts 的 `color` 数组）：随方案与模式变化。
 *
 * **跟随默认时返回空数组** —— 调用方据此「不覆盖」图表颜色，保持升级前的图表外观；
 * 已选方案时返回该方案推导出的 10 档图表色（与注入到 `--td-chart-color-*` 的值完全一致）。
 */
export function paletteChartColors(): string[] {
  const id = paletteId.value;
  if (!id) return [];
  const { vars } = buildPalette(id, currentMode());
  return Array.from({ length: 10 }, (_, index) => vars[`--td-chart-color-${index + 1}`]);
}
