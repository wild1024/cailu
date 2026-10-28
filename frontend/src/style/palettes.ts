/**
 * 配色方案引擎（8 套色板 × 明暗双模式）
 *
 * **默认不应用任何方案**：用户不选时本引擎不向页面注入任何变量，
 * `theme.less` 里的原有配色原样生效（升级零感知）；只有用户在
 * 「外观设置」或顶栏面板主动选择某套方案后才注入。
 *
 * 设计目标：**不为每套方案手写一份 theme.less**。
 * 每套方案只声明 5 个基准色（主色 / 浅色 / 交融色 / 渐染色 / 强调色），
 * 其余所有变量（brand 色阶 1~10、中性容器色、文字色、图表色）都在运行时推导，
 * 最终以 CSS 变量形式注入 `<html>` 的内联 style —— 内联样式优先级高于任何选择器，
 * 因此能压过 TDesign 的 `:root[theme-mode='light'|'dark']` 块（theme.less 顶部注释记的坑）。
 *
 * 变量名严格复用现有约定（`--td-brand-color-1..10` / `--td-bg-color-*` / `--td-text-color-*`
 * / `--td-chart-color-*` / `--app-*`），现有组件零改动即可换肤。
 *
 * ⚠ 色值红线：下面 PALETTES 里的 30 个色值来自需求方色表，**原样保留，不"优化"**。
 *   推导色阶、自动修正都是额外派生的变量，基准色本身永远不覆写。
 */

/** 明暗模式 */
export type ColorMode = 'light' | 'dark';

/** 方案 id（写入 localStorage，勿随意改） */
export type PaletteId =
  | 'mist-blue'
  | 'deep-sea'
  | 'teal-sunset'
  | 'rose-gold'
  | 'violet-honey'
  | 'jade-crimson'
  | 'glaze-teal'
  | 'inkberry';

/** 一套方案的基准色（5 个色值 + 文案） */
export interface PaletteDef {
  /** 稳定 id，持久化用 */
  id: PaletteId;
  /** 方案中文名 */
  name: string;
  /** 方案来源/气质说明，用于设置页副标题 */
  note: string;
  /** ① 主色 = brand-7 */
  brand: string;
  /** ② 浅色 = brand-5 */
  brandLight: string;
  /** ③ 交融色 = 页面底色 */
  merge: string;
  /** ④ 渐染色 = 边框 */
  border: string;
  /** ⑤ 强调色 accent */
  accent: string;
}

/**
 * 八套色板（前六套 + 釉青 / 墨莓）。
 *
 * 色块语义：
 * - brand   → `--td-brand-color-7`，主按钮 / 选中态 / 图表主色
 * - brandLight → `--td-brand-color-5`，浅色边框 / 次级强调
 * - merge   → `--td-bg-color-page`，页面底色（深色模式下压暗）
 * - border  → `--td-component-border`，卡片与分割线（深色模式下压暗）
 * - accent  → `--app-brand-accent` / `--td-chart-color-2`，点缀与图表第二色
 */
export const PALETTES: readonly PaletteDef[] = [
  {
    id: 'mist-blue',
    name: '雾蓝流金',
    note: '晨雾蓝 × 霜月白 × 落日流金',
    brand: '#6F8AB8',
    brandLight: '#A5B3CB',
    merge: '#F2F2EE',
    border: '#AAB7C2',
    accent: '#C1AC7B',
  },
  {
    id: 'deep-sea',
    name: '深海蓝',
    note: '深海蓝 × 雾蓝灰 × 漂木棕 —— 早期版本配色',
    brand: '#51697C',
    brandLight: '#AAB7C2',
    merge: '#F2F2EE',
    border: '#C6D0D8',
    accent: '#B79D88',
  },
  {
    id: 'teal-sunset',
    name: '青碧落日',
    note: '苔原青碧 × 晨光交融 × 落日橘金',
    brand: '#6FB8AA',
    brandLight: '#A5CBC3',
    merge: '#E4EAE1',
    border: '#CFB8AA',
    accent: '#C1AC7B',
  },
  {
    id: 'rose-gold',
    name: '玫紫流金',
    note: '霞紫 × 金玫交融 × 落日流金',
    brand: '#BA6DA7',
    brandLight: '#CCA4C2',
    merge: '#E8EAE1',
    border: '#D0C7A9',
    accent: '#C3B179',
  },
  {
    id: 'violet-honey',
    name: '黛紫蜜橘',
    note: '暮山黛紫 × 胭脂交融 × 暖蜜橘',
    brand: '#A26FB8',
    // ⚠ 需求原稿此处写作 `#BFS5CB`（含非法字符 S，不是合法十六进制）。
    //   按「主色同色相浅色」规律推断为 #BFA5CB（H≈281 与主色 #A26FB8 一致，L 抬高约 12%）。
    //   ★ 该值为**推断值**，拿到原稿修正后只需改这一处，其余全部自动重算。
    brandLight: '#BFA5CB',
    merge: '#EAE1E6',
    border: '#CFB8AA',
    accent: '#C1957B',
  },
  {
    id: 'jade-crimson',
    name: '翠绿绛红',
    note: '翠竹绿 × 朱砂交融 × 绛红渐染',
    brand: '#6FB885',
    brandLight: '#A5CBB0',
    merge: '#E8EAE1',
    border: '#CFAAAB',
    accent: '#C17B7E',
  },
  {
    id: 'glaze-teal',
    name: '釉青',
    note: '湖缥 × 银贝 × 墨青 —— 冷瓷清透',
    brand: '#7BA0A3',
    brandLight: '#A3BEBC',
    merge: '#ECEEEB',
    border: '#CDDDDF',
    accent: '#567076',
  },
  {
    id: 'inkberry',
    name: '墨莓',
    note: '绛莓 × 雪絮 × 夜绀 —— 沉静莓紫',
    brand: '#7D5266',
    brandLight: '#B299AB',
    merge: '#EDEBE9',
    border: '#D3C4CB',
    accent: '#484D61',
  },
] as const;

/**
 * 当前选择：8 套方案之一，或 `null`。
 *
 * `null` 的语义是「**跟随默认 / 保持原主题**」—— 不注入任何 CSS 变量，
 * `theme.less` 原样生效。这不是第 9 套方案，而是「不选择」这一状态本身。
 */
export type PaletteSelection = PaletteId | null;

/**
 * 默认值 = **不应用任何方案**（首次访问、清空 localStorage、点「跟随默认」都回到这里）。
 *
 * ⚠ 不要把它改成某个方案的 id：那会让所有老用户升级后界面立刻变样。
 */
export const DEFAULT_PALETTE_ID: PaletteSelection = null;

/** 是否为合法方案 id（localStorage 脏数据兜底用） */
export function isPaletteId(value: unknown): value is PaletteId {
  return typeof value === 'string' && PALETTES.some((item) => item.id === value);
}

/** 按 id 取方案；`null` / 非法 id 返回 `null`（表示「跟随默认」） */
export function findPalette(id: string | null | undefined): PaletteDef | null {
  if (!id) return null;
  return PALETTES.find((item) => item.id === id) ?? null;
}

/** 按 id 取方案；id 非法时回退到第一套（仅供 `buildPalette` 内部兜底，不表示"默认方案"） */
export function getPalette(id: string | null | undefined): PaletteDef {
  return PALETTES.find((item) => item.id === id) ?? PALETTES[0];
}

/* ============================================================================
 * 一、颜色基础运算（纯函数，无依赖，脚本与浏览器共用）
 * ========================================================================== */

/** RGB 三元组，分量 0~255 */
export interface Rgb {
  r: number;
  g: number;
  b: number;
}

/** HSL 三元组，h 0~360，s/l 0~100 */
export interface Hsl {
  h: number;
  s: number;
  l: number;
}

function clamp(value: number, min: number, max: number): number {
  return value < min ? min : value > max ? max : value;
}

/** 十六进制色值 → RGB；非法色值直接抛错（色表里出现过 `#BFS5CB` 这种脏数据） */
export function parseHex(hex: string): Rgb {
  const raw = hex.trim().replace(/^#/, '');
  const full =
    raw.length === 3
      ? raw
          .split('')
          .map((ch) => ch + ch)
          .join('')
      : raw;
  if (!/^[0-9a-fA-F]{6}$/.test(full)) {
    throw new Error(`非法十六进制色值：${hex}`);
  }
  return {
    r: parseInt(full.slice(0, 2), 16),
    g: parseInt(full.slice(2, 4), 16),
    b: parseInt(full.slice(4, 6), 16),
  };
}

/** RGB → `#rrggbb`（小写，便于断言比较） */
export function toHex(rgb: Rgb): string {
  const part = (value: number): string =>
    Math.round(clamp(value, 0, 255))
      .toString(16)
      .padStart(2, '0');
  return `#${part(rgb.r)}${part(rgb.g)}${part(rgb.b)}`;
}

/** RGB → HSL */
export function rgbToHsl(rgb: Rgb): Hsl {
  const r = rgb.r / 255;
  const g = rgb.g / 255;
  const b = rgb.b / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) {
    return { h: 0, s: 0, l: l * 100 };
  }
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h: number;
  if (max === r) {
    h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
  } else if (max === g) {
    h = ((b - r) / d + 2) / 6;
  } else {
    h = ((r - g) / d + 4) / 6;
  }
  return { h: h * 360, s: s * 100, l: l * 100 };
}

/** HSL → RGB */
export function hslToRgb(hsl: Hsl): Rgb {
  const h = ((hsl.h % 360) + 360) % 360 / 360;
  const s = clamp(hsl.s, 0, 100) / 100;
  const l = clamp(hsl.l, 0, 100) / 100;
  if (s === 0) {
    const v = l * 255;
    return { r: v, g: v, b: v };
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const hue = (t: number): number => {
    let tt = t;
    if (tt < 0) tt += 1;
    if (tt > 1) tt -= 1;
    if (tt < 1 / 6) return p + (q - p) * 6 * tt;
    if (tt < 1 / 2) return q;
    if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6;
    return p;
  };
  return { r: hue(h + 1 / 3) * 255, g: hue(h) * 255, b: hue(h - 1 / 3) * 255 };
}

/** 十六进制 → HSL */
export function hexToHsl(hex: string): Hsl {
  return rgbToHsl(parseHex(hex));
}

/** HSL → 十六进制 */
export function hslToHex(hsl: Hsl): string {
  return toHex(hslToRgb(hsl));
}

/** 只改明度与饱和度，色相锁定（色阶不漂移的关键） */
export function withL(hsl: Hsl, l: number, s?: number, h?: number): string {
  return hslToHex({ h: h ?? hsl.h, s: clamp(s ?? hsl.s, 0, 100), l: clamp(l, 0, 100) });
}

/** RGB 线性插值（t=0 取 a，t=1 取 b） */
export function blend(a: string, b: string, t: number): string {
  const ca = parseHex(a);
  const cb = parseHex(b);
  const k = clamp(t, 0, 1);
  return toHex({
    r: ca.r + (cb.r - ca.r) * k,
    g: ca.g + (cb.g - ca.g) * k,
    b: ca.b + (cb.b - ca.b) * k,
  });
}

/** WCAG 相对亮度 */
export function luminance(hex: string): number {
  const { r, g, b } = parseHex(hex);
  const channel = (value: number): number => {
    const v = value / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

/** WCAG 对比度（1~21） */
export function contrast(fg: string, bg: string): number {
  const lf = luminance(fg);
  const lb = luminance(bg);
  const lighter = Math.max(lf, lb);
  const darker = Math.min(lf, lb);
  return (lighter + 0.05) / (darker + 0.05);
}

/** 保留两位小数，报告输出用 */
export function round2(value: number): number {
  return Math.round(value * 100) / 100;
}

/**
 * 把前景色沿明度轴推到「与背景对比度 ≥ ratio」的最近位置。
 *
 * 背景偏亮 → 压暗前景；背景偏暗 → 提亮前景。逐 1% 明度步进，
 * 所以得到的一定是**最近的达标值**（需求里的"自动修正到最近达标值"）。
 * 极端情况下（背景压在中灰上）退化到纯黑/纯白兜底。
 */
export function ensureContrast(fg: string, bg: string, ratio: number): string {
  if (contrast(fg, bg) >= ratio) {
    return fg;
  }
  const hsl = hexToHsl(fg);
  const bgLum = luminance(bg);
  const darker = bgLum > 0.4;
  for (let step = 1; step <= 100; step += 1) {
    const l = darker ? hsl.l - step : hsl.l + step;
    if (l < 0 || l > 100) break;
    const candidate = withL(hsl, l);
    if (contrast(candidate, bg) >= ratio) {
      return candidate;
    }
  }
  return darker ? '#000000' : '#ffffff';
}

/* ============================================================================
 * 二、色阶推导
 * ========================================================================== */

/** 亮色模式：brand-1~4 的明度取值（L%），5 级起由色板给定的浅色接管 */
const LIGHT_TINTS = [96, 92, 87, 80];

/**
 * 生成亮色模式 brand 色阶 1~10。
 *
 * 两个锚点：`brand-5` 用色板给定的浅色（原值不动的浅色），`brand-7` 用色板主色。
 * 1~4 级在 brand-5 之上按明度阶梯铺开，6 级取 5/7 的明度中点，8~10 级按主色压暗。
 * 色相全程锁定主色色相，饱和度只在最浅的四级略微收敛（避免近白色发脏）。
 */
export function brandScaleLight(brand: string, brandLight: string): string[] {
  const main = hexToHsl(brand);
  const light = hexToHsl(brandLight);
  // 浅色必须比主色亮，否则色阶会乱序（脏数据兜底）
  const l5 = clamp(Math.max(light.l, main.l + 6), 55, 82);
  const ladder = [
    LIGHT_TINTS[0],
    LIGHT_TINTS[1],
    LIGHT_TINTS[2],
    LIGHT_TINTS[3],
    l5,
    (l5 + main.l) / 2,
    main.l,
    main.l - 9,
    main.l - 18,
    main.l - 27,
  ];
  return ladder.map((l, index) => {
    if (index === 4) {
      return light.l === l5 ? brandLight.toLowerCase() : withL(main, l, main.s);
    }
    if (index === 6) {
      return brand.toLowerCase();
    }
    // 1~3 级做轻微去饱和，贴近底色时不刺眼
    const s = index <= 2 ? main.s * 0.72 : index === 3 ? main.s * 0.86 : main.s;
    return withL(main, clamp(l, 4, 99), s);
  });
}

/**
 * 生成深色模式 brand 色阶 1~10。
 *
 * 与 TDesign 深色语义一致：1 级最深（选中底/浅面），7 级主色，8~10 级比主色更亮。
 * 主色必须**提亮**（深底上原色会糊掉），提亮幅度取 +14 L，并保证与深色页面底
 * 至少 3.0 的对比度（UI 组件可辨底线）。
 */
export function brandScaleDark(brand: string, pageBg: string): string[] {
  const main = hexToHsl(brand);
  let l7 = clamp(main.l + 14, 44, 62);
  let mainDark = withL(main, l7);
  // 深底可辨底线：不足 3.0 继续提亮（最多到 78）
  while (contrast(mainDark, pageBg) < 3 && l7 < 78) {
    l7 += 1;
    mainDark = withL(main, l7);
  }
  const ladder = [l7 - 32, l7 - 27, l7 - 22, l7 - 17, l7 - 12, l7 - 6, l7, l7 + 7, l7 + 14, l7 + 21];
  return ladder.map((l, index) => withL(main, clamp(l, 10, 92), index < 6 ? main.s * 0.92 : main.s));
}

/* ============================================================================
 * 三、整套变量生成（含对比度自动修正与告警）
 * ========================================================================== */

/** 单条对比度检查结果 */
export interface ContrastCheck {
  /** 检查项键（报告表头用） */
  key: string;
  /** 人类可读标签 */
  label: string;
  /** 前景色 */
  fg: string;
  /** 背景色 */
  bg: string;
  /** 实测对比度 */
  ratio: number;
  /** 门槛 */
  min: number;
  /** 是否达标 */
  ok: boolean;
  /** 是否因为不达标被自动修正过 */
  adjusted: boolean;
}

/** 一套方案的完整产物 */
export interface PaletteBuild {
  /** 方案 */
  palette: PaletteDef;
  /** 模式 */
  mode: ColorMode;
  /** 变量表（变量名 → 值） */
  vars: Record<string, string>;
  /** 对比度检查 */
  checks: ContrastCheck[];
  /** 推过程中的告警（自动修正、色值推断等） */
  warnings: string[];
}

/** 10 档图表色的色相偏移（0 = 主色本身，其余围绕主色展开） */
const CHART_HUE_OFFSETS = [0, 26, -28, 58, 128, 186, 232, 286, 322, 94];

/**
 * 生成某套方案 × 某个模式的全部 CSS 变量。
 *
 * 浅色：底色用交融色；容器色向白色提亮；文字色由主色色相压暗并做 AA 修正。
 * 深色：交融色压到 L≈11% 作页面底；容器色向上提亮一档；文字色提亮并做 AA 修正；
 *       品牌色提亮（见 brandScaleDark），颜色永远不跨模式复用。
 */
export function buildPalette(paletteId: string, mode: ColorMode): PaletteBuild {
  const palette = getPalette(paletteId);
  const warnings: string[] = [];
  const mainHsl = hexToHsl(palette.brand);
  const mergeHsl = hexToHsl(palette.merge);
  const borderHsl = hexToHsl(palette.border);

  // ---------- 页面底色与中性色面 ----------
  let page: string;
  let container: string;
  let secondaryContainer: string;
  let component: string;
  let specialComponent: string;
  let stroke: string;
  let borderColor: string;

  if (mode === 'light') {
    page = palette.merge.toLowerCase();
    // 容器向白色提亮 62%：保证卡片比页面底更"白"，层次来自明度而不是灰
    container = blend(palette.merge, '#ffffff', 0.62);
    secondaryContainer = blend(container, palette.merge, 0.34);
    component = blend(container, palette.merge, 0.2);
    specialComponent = blend(container, '#ffffff', 0.4);
    stroke = blend(palette.border, container, 0.42);
    borderColor = palette.border.toLowerCase();
  } else {
    // 交融色压暗到 L≈11%，保留一点原色相，深底才有"这套方案"的味道
    page = withL(mergeHsl, 11, clamp(mergeHsl.s * 1.6, 6, 26), mainHsl.h);
    container = withL(mergeHsl, 16, clamp(mergeHsl.s * 1.3, 4, 18), mainHsl.h);
    secondaryContainer = withL(mergeHsl, 21, clamp(mergeHsl.s, 3, 14), mainHsl.h);
    component = withL(mergeHsl, 24, clamp(mergeHsl.s, 3, 12), mainHsl.h);
    specialComponent = withL(mergeHsl, 13, clamp(mergeHsl.s, 4, 16), mainHsl.h);
    // 深色模式下边框要比底亮一点才看得见
    stroke = withL(borderHsl, 26, clamp(borderHsl.s * 0.7, 4, 18), mainHsl.h);
    borderColor = withL(borderHsl, 34, clamp(borderHsl.s * 0.8, 5, 20), mainHsl.h);
  }
  // 悬停/激活：浅色是往底色方向压一点，深色是往白色方向提一点（深底"越交互越亮"）
  const containerHover = mode === 'light' ? blend(container, page, 0.32) : blend(container, '#ffffff', 0.06);
  const containerActive = mode === 'light' ? blend(container, page, 0.5) : blend(container, '#ffffff', 0.1);

  // ---------- 文字色：由主色色相压暗 / 提亮，并逐项做 AA 修正 ----------
  const textSeedHue = mainHsl.h;
  const textSeedSat = clamp(mainHsl.s * 0.28, 6, 22);
  const textSeedL = mode === 'light' ? 20 : 92;
  const textPrimaryRaw = hslToHex({ h: textSeedHue, s: textSeedSat, l: textSeedL });
  const textSecondaryRaw = hslToHex({ h: textSeedHue, s: textSeedSat, l: mode === 'light' ? 33 : 76 });
  const textPlaceholderRaw = hslToHex({ h: textSeedHue, s: textSeedSat, l: mode === 'light' ? 52 : 58 });

  const textPrimary = ensureContrast(textPrimaryRaw, page, 7);
  const textSecondary = ensureContrast(textSecondaryRaw, page, 4.5);
  // 占位/空状态文字：TDesign 还会把它用在表头、面包屑、descriptions 标签上，
  // 而这些场景的底色是「容器」（比页面底更亮/更暗），所以页面底与容器底都要 ≥3.0
  const textPlaceholder = ensureContrast(ensureContrast(textPlaceholderRaw, page, 3), container, 3);
  // 禁用/空状态文字：TDesign 的「暂无数据」也走这一档，所以同样按容器底 ≥3.0 兜底
  const textDisabled = ensureContrast(
    ensureContrast(hslToHex({ h: textSeedHue, s: textSeedSat, l: mode === 'light' ? 68 : 44 }), page, 2),
    container,
    3,
  );

  // ---------- brand 色阶 ----------
  const scaleLight = brandScaleLight(palette.brand, palette.brandLight);
  const scale =
    mode === 'light' ? scaleLight : brandScaleDark(palette.brand, page);
  const brandMain = scale[6];
  // 强调色/图表色的可见度底线：WCAG 1.4.11 对"有意义的图形"要求 ≥3，这里取 2.5（色块面积大的折线/柱）
  const SURFACE_MIN = 2.5;
  const accentSafe = ensureContrast(palette.accent, page, SURFACE_MIN);

  // ---------- 品牌文字色（链接 / 选中文字）：页面底与容器底都要 AA ----------
  // 链接既可能落在页面底，也可能落在卡片（容器底）上，两处都要达标，
  // 因此连续修正两次（第二次只会朝同方向继续走，不会破坏第一次的结果）。
  const brandText = ensureContrast(ensureContrast(brandMain, page, 4.5), container, 4.5);

  // ---------- 主交互面与「主色上的文字」----------
  // TDesign 的 `--td-brand-color` 一变量两用：既是主按钮底色，又是链接/选中页签的文字色。
  // 两种用法对明度的要求相反（底要深才配白字，字要亮才在深底上可辨），所以：
  //   · 浅色模式：白字不达标时，主交互面下移到压暗版（满足"白字按钮可读"），压暗后的颜色
  //     在浅底上作文字也天然达标；
  //   · 深色模式：主交互面保持**提亮后**的主色（要求 2：深底上品牌色必须提亮），
  //     白字改由 `--app-on-brand`（深色油墨）承担 —— 白底字改成深底字，两种模式都 ≥ 4.5。
  const whiteOnBrand = contrast('#ffffff', brandMain);
  const brandStrong = ensureContrast(brandMain, '#ffffff', 4.5);
  const brandStrongNeeded = whiteOnBrand < 4.5;
  if (brandStrongNeeded) {
    warnings.push(
      `${palette.name}（${mode === 'light' ? '浅色' : '深色'}）：白字在主色 ${brandMain} 上对比度 ${round2(
        whiteOnBrand,
      )} < 4.5，已派生压暗版 --app-brand-strong = ${brandStrong}（白字对比度 ${round2(
        contrast('#ffffff', brandStrong),
      )}）；${
        mode === 'light'
          ? '主按钮底色自动下移到压暗版'
          : '深色模式下按钮文字改用 --app-on-brand 深色油墨'
      }`,
    );
  }

  // 主交互面统一用压暗版：深色模式下若仍用主色，白字与深油墨都可能压不到 4.5
  //（例：墨莓深色主色 #a3738a 白字仅 3.91、深油墨上限约 4.3）。
  const brandSurface = brandStrongNeeded ? brandStrong : brandMain;
  // 主色上的文字色：优先白字，白字不达标则用与主色同色相的深色油墨
  const onBrand =
    contrast('#ffffff', brandSurface) >= 4.5
      ? '#ffffff'
      : ensureContrast(hslToHex({ h: mainHsl.h, s: clamp(mainHsl.s, 24, 62), l: 12 }), brandSurface, 4.5);
  const surfaceHsl = hexToHsl(brandSurface);
  // 深色模式"越交互越亮"，浅色模式"越交互越深"
  const brandSurfaceHover =
    mode === 'dark' ? withL(surfaceHsl, surfaceHsl.l + 8) : withL(surfaceHsl, surfaceHsl.l - 6);
  const brandSurfaceActive =
    mode === 'dark' ? withL(surfaceHsl, surfaceHsl.l + 14) : withL(surfaceHsl, surfaceHsl.l - 12);
  // 品牌色落在「品牌浅底块」(brand-1) 上时的文字色：深色模式下 brand-1 是一块偏暗的面，
  // 直接用 brand-7 当文字会糊（实测 1.5 左右），必须再提亮一档。
  const onBrandTint = ensureContrast(brandMain, scale[0], 4.5);

  // 品牌渐变：起点用压暗版、中段再按白字 4.5 兜底 —— 渐变条上的白字必须能读
  // ── 横幅渐变（老大 2026-10-28 第三版要求）──
  // 起点 = 方案主色，终点 = 方案第 2 色（同色系较浅那个 palette.brandLight）。
  // 老大两次反馈"前段太浅"：最终方案 = 前 72% 纯主色，只在最后 28% 过渡到方案浅色。
  // 因为终点是浅色，白字在上面只有约 2.1，所以渐变上的文字改用同色相极深油墨 --app-on-gradient，
  // 并做两轮 ensureContrast：分别对起点(主色)与终点(浅色)都保证 ≥ 4.5。
  // 起点用压暗版 brandStrong：白字在其上恒 ≥4.5，且前段视觉更深（老大反馈"太浅"）。
  // 前 38% 保持纯压暗版 —— 标题/副标题都落在这段里，保证白字可读（青碧/翠绿主色太浅，
  // 若文字跨进主色段会掉到 2.3 不可读）。
  const gradientStart = brandStrong.toLowerCase();
  const gradientMid = palette.brand.toLowerCase();
  const gradientEnd = palette.brandLight.toLowerCase();
  // 渐变文字用白字：起点是压暗版 brandStrong（白字 ≥4.5），前 55% 区域文字均落在深色段。
  // 后段渐浅处不放文字（副标题也左对齐），浅色端只留图标。
  const onGradient = '#ffffff';
  // 图标/浅色端用的深色油墨（浅色段上白字不可见）
  const onGradientSoft = withL(hexToHsl(palette.brand), 12, 30);

  // ---------- 图表色 ----------
  // 图表是"有意义的图形"，按 WCAG 1.4.11 不低于 2.5 的可见度底线；
  // 主色本身在亮底太浅时（青碧 / 翠绿）自动压深一档并告警，色相不变。
  const CHART_MIN = SURFACE_MIN;
  const chartPrimary = ensureContrast(brandMain, page, CHART_MIN);
  if (chartPrimary !== brandMain) {
    warnings.push(
      `${palette.name}（${mode === 'light' ? '浅色' : '深色'}）：主色在页面底上仅 ${round2(
        contrast(brandMain, page),
      )} < ${CHART_MIN}，图表主色自动修正为 ${chartPrimary}（--td-brand-color-7 保持原值）`,
    );
  }
  const chartSoftL = mode === 'light' ? mainHsl.l + 18 : hexToHsl(brandMain).l + 12;
  const chartColors = CHART_HUE_OFFSETS.map((offset, index) => {
    if (index === 0) return chartPrimary;
    if (index === 1) return accentSafe;
    const hue = mainHsl.h + offset;
    const sat = clamp(mainHsl.s * 0.92, 18, 78);
    const lightness =
      index % 3 === 0 ? clamp(chartSoftL, 30, 86) : index % 3 === 1 ? clamp(mainHsl.l - 6, 16, 78) : clamp(hexToHsl(brandMain).l, 18, 82);
    return ensureContrast(hslToHex({ h: hue, s: sat, l: lightness }), page, CHART_MIN);
  });

  // ---------- 组装变量表 ----------
  const vars: Record<string, string> = {
    // brand 色阶
    '--td-brand-color-1': scale[0],
    '--td-brand-color-2': scale[1],
    '--td-brand-color-3': scale[2],
    '--td-brand-color-4': scale[3],
    '--td-brand-color-5': scale[4],
    '--td-brand-color-6': scale[5],
    '--td-brand-color-7': scale[6],
    '--td-brand-color-8': scale[7],
    '--td-brand-color-9': scale[8],
    '--td-brand-color-10': scale[9],
    // brand 派生语义（主交互面 = AA 安全色）
    '--td-brand-color': brandSurface,
    '--td-brand-color-light': scale[0],
    '--td-brand-color-focus': scale[1],
    '--td-brand-color-hover': brandSurfaceHover,
    '--td-brand-color-active': brandSurfaceActive,
    '--td-brand-color-disabled': scale[2],
    '--td-brand-color-light-hover': scale[1],
    '--td-brand-color-fade': scale[0],
    '--td-brand-color-focus-light': scale[0],

    // 背景 / 容器
    '--td-bg-color-page': page,
    '--td-bg-color-container': container,
    '--td-bg-color-container-hover': containerHover,
    '--td-bg-color-container-active': containerActive,
    '--td-bg-color-container-select': mode === 'light' ? scale[0] : scale[1],
    '--td-bg-color-secondarycontainer': secondaryContainer,
    '--td-bg-color-secondarycontainer-hover':
      mode === 'light' ? blend(secondaryContainer, page, 0.4) : blend(secondaryContainer, '#ffffff', 0.06),
    '--td-bg-color-secondarycontainer-active':
      mode === 'light' ? blend(secondaryContainer, page, 0.6) : blend(secondaryContainer, '#ffffff', 0.1),
    '--td-bg-color-component': component,
    '--td-bg-color-component-hover':
      mode === 'light' ? blend(component, page, 0.4) : blend(component, '#ffffff', 0.08),
    '--td-bg-color-component-active':
      mode === 'light' ? blend(component, page, 0.6) : blend(component, '#ffffff', 0.12),
    '--td-bg-color-component-disabled':
      mode === 'light' ? blend(component, page, 0.55) : blend(component, page, 0.45),
    '--td-bg-color-secondarycomponent': mode === 'light' ? blend(component, page, 0.3) : component,
    '--td-bg-color-secondarycomponent-hover':
      mode === 'light' ? blend(component, page, 0.5) : blend(component, '#ffffff', 0.1),
    '--td-bg-color-secondarycomponent-active':
      mode === 'light' ? blend(component, page, 0.65) : blend(component, '#ffffff', 0.16),
    '--td-bg-color-specialcomponent': specialComponent,

    // 描边
    '--td-component-stroke': stroke,
    '--td-component-border': borderColor,

    // 文字
    '--td-text-color-primary': textPrimary,
    '--td-text-color-secondary': textSecondary,
    '--td-text-color-placeholder': textPlaceholder,
    '--td-text-color-disabled': textDisabled,
    '--td-text-color-brand': brandText,
    '--td-text-color-link': brandText,
    '--td-text-color-anti': '#ffffff',

    // 图表
    '--td-chart-color': chartColors[0],
    ...Object.fromEntries(chartColors.map((color, index) => [`--td-chart-color-${index + 1}`, color])),

    // 投影：带一点主色，和整体色调呼应
    '--td-shadow-1': `0 1px 10px ${rgba(brandMain, 0.1)}`,
    '--td-shadow-2': `0 3px 6px -4px ${rgba(brandMain, 0.14)}, 0 6px 16px ${rgba(brandMain, 0.1)}`,
    '--td-shadow-3': `0 6px 30px 5px ${rgba(brandMain, 0.08)}, 0 16px 24px 2px ${rgba(brandMain, 0.06)}`,

    // 业务别名（现有代码里已经在用的 --app-* 全部覆盖）
    '--app-brand': brandSurface,
    '--app-brand-hover': brandSurfaceHover,
    '--app-brand-active': brandSurfaceActive,
    '--app-brand-light': scale[0],
    '--app-brand-border': scale[3],
    '--app-brand-accent': accentSafe,
    '--app-brand-strong': brandStrong,
    '--app-brand-strong-hover': withL(hexToHsl(brandStrong), hexToHsl(brandStrong).l - 6),
    '--app-brand-strong-active': withL(hexToHsl(brandStrong), hexToHsl(brandStrong).l - 12),
    // 主色**上面**的文字色：白字达标就是白，否则是同色相深油墨（深色模式下浅品牌底必须配深字）
    '--app-on-brand': onBrand,
    // 主色**浅底块**（brand-1）上的品牌文字色
    '--app-brand-on-tint': onBrandTint,
    '--app-palette-brand': palette.brand.toLowerCase(),
    '--app-palette-brand-light': palette.brandLight.toLowerCase(),
    '--app-palette-merge': palette.merge.toLowerCase(),
    '--app-palette-border': palette.border.toLowerCase(),
    '--app-palette-accent': palette.accent.toLowerCase(),
    '--app-bg-page': page,
    '--app-bg-container': container,
    '--app-border': borderColor,
    '--app-gradient-brand': `linear-gradient(135deg, ${gradientStart} 0%, ${gradientStart} 38%, ${gradientMid} 64%, ${gradientEnd} 100%)`,
    '--app-on-gradient': onGradient,
    '--app-on-gradient-soft': onGradientSoft,
    // 色卡 / 预览区用：5 个基准色原值（不做任何修正，永远等于需求色表）
    '--app-swatch-1': palette.brand.toLowerCase(),
    '--app-swatch-2': palette.brandLight.toLowerCase(),
    '--app-swatch-3': palette.merge.toLowerCase(),
    '--app-swatch-4': palette.border.toLowerCase(),
    '--app-swatch-5': palette.accent.toLowerCase(),
  };

  // ---------- 对比度检查（报告 = 本功能的测试证据） ----------
  const checks: ContrastCheck[] = [
    makeCheck('text-primary', '正文 / 页面底', textPrimary, page, 4.5, textPrimary !== textPrimaryRaw),
    makeCheck('text-secondary', '次要文字 / 页面底', textSecondary, page, 4.5, textSecondary !== textSecondaryRaw),
    makeCheck('text-placeholder', '占位文字 / 页面底', textPlaceholder, page, 3, textPlaceholder !== textPlaceholderRaw),
    makeCheck('text-placeholder-on-container', '占位文字 / 容器底', textPlaceholder, container, 3, textPlaceholder !== textPlaceholderRaw),
    makeCheck('text-on-container', '正文 / 容器底', textPrimary, container, 4.5, false),
    makeCheck('text-secondary-on-container', '次要文字 / 容器底', textSecondary, container, 4.5, false),
    makeCheck('brand-text', '品牌文字 / 页面底', brandText, page, 4.5, brandText !== brandMain),
    makeCheck('brand-text-on-container', '品牌文字 / 容器底', brandText, container, 4.5, brandText !== brandMain),
    makeCheck('white-on-brand-label', '白字 / 主色（brand-7）', '#ffffff', brandMain, 4.5, false),
    makeCheck('on-brand-surface', '主色上的文字 / 主交互面', onBrand, brandSurface, 4.5, brandStrongNeeded),
    makeCheck('brand-on-tint', '品牌文字 / 品牌浅底块', onBrandTint, scale[0], 4.5, onBrandTint !== brandMain),
    makeCheck('chart-1-on-page', '图表主色 / 页面底', chartPrimary, page, 2.5, chartPrimary !== brandMain),
    makeCheck('chart-2-on-page', '图表强调色 / 页面底', chartColors[1], page, 2.5, accentSafe !== palette.accent),
    // 文字区锁在 0~38% 的纯压暗版段（白字 ≥4.5）；主色段(38~64%)与浅色端只作视觉过渡，
    // 浅色段若放文字则由深色油墨承担（如图标）。
    makeCheck('on-gradient-start', '白字 / 渐变起点(压暗版)', onGradient, gradientStart, 4.5, true),
    makeCheck('on-gradient-end', '深油墨 / 渐变终点(浅色)', onGradientSoft, gradientEnd, 4.5, true),
  ];

  return { palette, mode, vars, checks, warnings };
}

function makeCheck(
  key: string,
  label: string,
  fg: string,
  bg: string,
  min: number,
  adjusted: boolean,
): ContrastCheck {
  const ratio = round2(contrast(fg, bg));
  return { key, label, fg, bg, ratio, min, ok: ratio >= min, adjusted };
}

/** 转成 CSS `rgb(r g b / a)`（现代语法，TDesign 同款） */
export function rgba(hex: string, alpha: number): string {
  const { r, g, b } = parseHex(hex);
  return `rgb(${r} ${g} ${b} / ${Math.round(alpha * 100)}%)`;
}

/** 变量表 → CSS 声明块文本（`--x: y;` 每行一条） */
export function varsToCss(vars: Record<string, string>, selector = ':root'): string {
  const body = Object.entries(vars)
    .map(([name, value]) => `  ${name}: ${value};`)
    .join('\n');
  return `${selector} {\n${body}\n}\n`;
}

/** 全部 8×2 组合的构建结果（脚本与设置页共用） */
export function buildAllPalettes(): PaletteBuild[] {
  const list: PaletteBuild[] = [];
  for (const palette of PALETTES) {
    for (const mode of ['light', 'dark'] as ColorMode[]) {
      list.push(buildPalette(palette.id, mode));
    }
  }
  return list;
}
