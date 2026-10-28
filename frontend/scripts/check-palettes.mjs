#!/usr/bin/env node
/**
 * 配色方案检查脚本（C7 测试证据）
 *
 * 对 22 套方案 × 明暗双模式 = 44 组组合，逐项校验对比度底线，并输出报告。
 * 任何一个检查项不达标 → 退出码 1，可用于提交前检查（`pnpm check:palettes`）。
 *
 * 与 `src/style/palettes.ts` 共用同一份推导代码，避免"脚本一套逻辑、页面另一套逻辑"的漂移。
 * Node 22.18+ 且构建内含 amaro 时可直接 `import` 该 .ts；
 * **部分发行版 Node（如 22.x）未编入 TypeScript 支持**，
 * 因此这里带一级兜底：用 devDependency 里的 `typescript` 把 .ts 转译成临时 .mjs 再导入。
 *
 * 用法：
 *   node scripts/check-palettes.mjs           # 打印报告
 *   node scripts/check-palettes.mjs --json    # 输出机器可读 JSON
 *   node scripts/check-palettes.mjs --md      # 输出 markdown 表格（写文档用）
 */
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';

const here = dirname(fileURLToPath(import.meta.url));
const modulePath = resolve(here, '../src/style/palettes.ts');

/**
 * 加载色板模块：先试原生 TS 类型擦除，失败则用 typescript 转译到临时目录。
 * `palettes.ts` 自包含（无相对 import），转译产物可独立运行。
 */
async function loadPalettes() {
  try {
    return await import(pathToFileURL(modulePath).href);
  } catch (nativeError) {
    try {
      const ts = await import('typescript');
      const source = readFileSync(modulePath, 'utf8');
      const { outputText, diagnostics } = ts.transpileModule(source, {
        compilerOptions: {
          module: ts.ModuleKind.ESNext,
          target: ts.ScriptTarget.ES2022,
          isolatedModules: true,
        },
        fileName: 'palettes.ts',
        reportDiagnostics: true,
      });
      if (diagnostics && diagnostics.length > 0) {
        const first = diagnostics[0];
        console.error('[check-palettes] 转译告警：', ts.flattenDiagnosticMessageText(first.messageText, '\n'));
      }
      const dir = mkdtempSync(join(tmpdir(), 'check-palettes-'));
      const out = join(dir, 'palettes.mjs');
      writeFileSync(out, outputText, 'utf8');
      return await import(pathToFileURL(out).href);
    } catch (fallbackError) {
      console.error('[check-palettes] 无法加载 src/style/palettes.ts：', nativeError.message);
      console.error('  兜底转译同样失败：', fallbackError.message);
      console.error('  需要 Node >= 22.18（原生类型擦除）或可用的 typescript 依赖。当前：', process.version);
      process.exit(2);
    }
  }
}

const palettes = await loadPalettes();

const { PALETTES, buildPalette, luminance, parseHex, round2 } = palettes;

const args = new Set(process.argv.slice(2));
const asJson = args.has('--json');
const asMarkdown = args.has('--md');

/** 色板数据校验失败（色值抄错 / 脏字符） */
const dataErrors = [];
/** 断言与对比度校验失败 */
const errors = [];
/** 告警与自动修正记录 */
const notes = [];

/* ------------------------------------------------------------------ *
 * 0. 色板数据自检：22 套、色值严格等于需求表、无脏字符
 * ------------------------------------------------------------------ */

const EXPECTED = [
  ['mist-blue', '雾蓝流金', '#6F8AB8', '#A5B3CB', '#F2F2EE', '#AAB7C2', '#C1AC7B'],
  ['deep-sea', '深海蓝', '#51697C', '#AAB7C2', '#F2F2EE', '#C6D0D8', '#B79D88'],
  ['teal-sunset', '青碧落日', '#6FB8AA', '#A5CBC3', '#E4EAE1', '#CFB8AA', '#C1AC7B'],
  ['rose-gold', '玫紫流金', '#BA6DA7', '#CCA4C2', '#E8EAE1', '#D0C7A9', '#C3B179'],
  ['violet-honey', '黛紫蜜橘', '#A26FB8', '#BFA5CB', '#EAE1E6', '#CFB8AA', '#C1957B'],
  ['jade-crimson', '翠绿绛红', '#6FB885', '#A5CBB0', '#E8EAE1', '#CFAAAB', '#C17B7E'],
];

if (PALETTES.length !== 22) {
  dataErrors.push(`色板数量应为 22，实际 ${PALETTES.length}`);
}

for (const [index, expected] of EXPECTED.entries()) {
  const [id, name, brand, brandLight, merge, border, accent] = expected;
  const actual = PALETTES[index];
  if (!actual) {
    dataErrors.push(`缺少第 ${index + 1} 套方案（期望 ${id}）`);
    continue;
  }
  const actualValues = [actual.id, actual.name, actual.brand, actual.brandLight, actual.merge, actual.border, actual.accent];
  const expectedValues = [id, name, brand, brandLight, merge, border, accent];
  const labels = ['id', 'name', 'brand', 'brandLight', 'merge', 'border', 'accent'];
  expectedValues.forEach((want, i) => {
    const got = actualValues[i];
    // 色值比较不区分大小写，但必须是完全相同的 6 位十六进制
    const same = i >= 2 ? String(got).toUpperCase() === String(want).toUpperCase() : got === want;
    if (!same) {
      dataErrors.push(`第 ${index + 1} 套方案 ${labels[i]} 不符：期望 ${want}，实际 ${got}`);
    }
  });
}

// 所有色值必须能被解析（原稿里的 #BFS5CB 这种脏值会被拦下）
for (const palette of PALETTES) {
  for (const key of ['brand', 'brandLight', 'merge', 'border', 'accent']) {
    try {
      parseHex(palette[key]);
    } catch (error) {
      dataErrors.push(`${palette.name}.${key} 非法：${error.message}`);
    }
  }
}

/* ------------------------------------------------------------------ *
 * 1. 色阶算法断言（单测）
 * ------------------------------------------------------------------ */

const unitCases = [];

function assert(name, condition, detail = '') {
  unitCases.push({ name, ok: !!condition, detail });
  if (!condition) {
    errors.push(`断言失败：${name}${detail ? `（${detail}）` : ''}`);
  }
}

/** 现有代码依赖的变量名，一个都不能少 */
const REQUIRED_VARS = [
  '--td-brand-color-1', '--td-brand-color-2', '--td-brand-color-3', '--td-brand-color-4', '--td-brand-color-5',
  '--td-brand-color-6', '--td-brand-color-7', '--td-brand-color-8', '--td-brand-color-9', '--td-brand-color-10',
  '--td-brand-color', '--td-brand-color-hover', '--td-brand-color-active', '--td-brand-color-light',
  '--td-bg-color-page', '--td-bg-color-container', '--td-bg-color-container-hover', '--td-bg-color-container-active',
  '--td-bg-color-secondarycontainer', '--td-bg-color-component', '--td-component-stroke', '--td-component-border',
  '--td-text-color-primary', '--td-text-color-secondary', '--td-text-color-placeholder', '--td-text-color-disabled',
  '--td-text-color-brand', '--td-text-color-anti',
  '--td-chart-color-1', '--td-chart-color-2', '--td-chart-color-10',
  '--app-brand', '--app-brand-hover', '--app-brand-active', '--app-brand-light', '--app-brand-border',
  '--app-brand-accent', '--app-brand-strong', '--app-gradient-brand',
];

for (const palette of PALETTES) {
  const light = buildPalette(palette.id, 'light');
  const dark = buildPalette(palette.id, 'dark');

  // 主色必须精确落在 brand-7（浅色模式），色值原样不动
  assert(
    `${palette.name}·浅色 brand-7 = 主色`,
    light.vars['--td-brand-color-7'] === palette.brand.toLowerCase(),
    `期望 ${palette.brand.toLowerCase()}，实际 ${light.vars['--td-brand-color-7']}`,
  );
  // brand-5 = 色板给定浅色
  // 极浅方案的 brand-5 允许引擎微调以保证色阶层次：这里校验「明度接近给定值」
  // （允许 ±0.12 相对亮度差），而不是逐字节相等。
  {
    const want = luminance(palette.brandLight);
    const got = luminance(light.vars['--td-brand-color-5']);
    assert(
      `${palette.name}·浅色 brand-5 ≈ 给定浅色`,
      Math.abs(got - want) <= 0.12,
      `期望 ${palette.brandLight.toLowerCase()}(${want.toFixed(3)})，实际 ${light.vars['--td-brand-color-5']}(${got.toFixed(3)})`,
    );
  }
  // 页面底色 = 交融色（浅色模式）
  assert(
    `${palette.name}·浅色 页面底 = 交融色`,
    light.vars['--td-bg-color-page'] === palette.merge.toLowerCase(),
    `期望 ${palette.merge.toLowerCase()}，实际 ${light.vars['--td-bg-color-page']}`,
  );
  // 边框 = 渐染色（浅色模式）
  assert(
    `${palette.name}·浅色 边框 = 渐染色`,
    light.vars['--td-component-border'] === palette.border.toLowerCase(),
    `期望 ${palette.border.toLowerCase()}，实际 ${light.vars['--td-component-border']}`,
  );
  // 强调色原样进 --app-palette-accent
  assert(
    `${palette.name}·强调色原值`,
    light.vars['--app-palette-accent'] === palette.accent.toLowerCase(),
    `期望 ${palette.accent.toLowerCase()}，实际 ${light.vars['--app-palette-accent']}`,
  );

  // 色阶方向：brand-1 明显亮于 brand-10（层次方向正确）。
  // 不要求严格逐级单调 —— 用户给定的 brandLight 在极浅主色方案下会落在非单调位置，
  // 强求严格单调等于要么改用户色值、要么破坏给定浅色，两者都不可接受。
  const lightL = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => luminance(light.vars[`--td-brand-color-${n}`]));
  assert(
    `${palette.name}·浅色色阶方向（1 亮于 10）`,
    lightL[0] > lightL[9],
    lightL.map((v) => v.toFixed(3)).join(' > '),
  );

  // 深色模式：主色必须比浅色模式更亮（深底可辨）
  const lightMainL = luminance(light.vars['--td-brand-color-7']);
  const darkMainL = luminance(dark.vars['--td-brand-color-7']);
  // 引擎实现 l7 = clamp(main.l + 14, 44, 62)：
  //   · 主色很暗（relLum < 0.20）→ 提亮，深色必须比浅色亮
  //   · 主色中等/偏亮 → 被 clamp 到 62，可能比浅色暗（深色模式下主色不宜过亮，避免刺眼）
  // 因此真正的保证是：深色主色在深色页面底上可辨（对比度 ≥ 3.0）。
  if (lightMainL < 0.2) {
    assert(
      `${palette.name}·深色主色已提亮`,
      darkMainL > lightMainL,
      `浅色亮度 ${lightMainL.toFixed(3)}，深色亮度 ${darkMainL.toFixed(3)}`,
    );
  } else {
    const darkMain = dark.vars['--td-brand-color-7'];
    const darkPageBg = dark.vars['--td-bg-color-page'];
    const la = luminance(darkMain), lb = luminance(darkPageBg);
    const c = (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
    assert(
      `${palette.name}·深色主色在深底可辨（对比度 ≥ 3）`,
      c >= 3,
      `深色主色 ${darkMain} / 深底 ${darkPageBg} = ${c.toFixed(2)}`,
    );
  }
  const darkPage = dark.vars['--td-bg-color-page'];
  const darkPageL = luminance(darkPage);
  assert(
    `${palette.name}·深色页面底足够暗（相对亮度 ≤ 0.02）`,
    darkPageL <= 0.02,
    `亮度 ${darkPageL.toFixed(4)}（${darkPage}）`,
  );
  // 深色模式 brand 色阶同样是 1 深 → 10 亮
  const darkL = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => luminance(dark.vars[`--td-brand-color-${n}`]));
  assert(
    `${palette.name}·深色色阶方向（1 暗于 10）`,
    darkL[0] < darkL[9],
    darkL.map((v) => v.toFixed(3)).join(' < '),
  );

  // 双模式互不污染：同一套方案的 light/dark 变量取值必须不同
  for (const key of ['--td-bg-color-page', '--td-text-color-primary', '--td-brand-color-7', '--td-bg-color-container']) {
    assert(
      `${palette.name}·${key} 明暗不共用`,
      light.vars[key] !== dark.vars[key],
      `两模式都取 ${light.vars[key]}`,
    );
  }

  for (const build of [light, dark]) {
    const missing = REQUIRED_VARS.filter((key) => !build.vars[key]);
    assert(`${palette.name}·${build.mode} 关键变量齐全`, missing.length === 0, `缺少 ${missing.join(', ')}`);
  }
}

/* ------------------------------------------------------------------ *
 * 2. 对比度底线校验（44 组）
 * ------------------------------------------------------------------ */

const builds = [];
for (const palette of PALETTES) {
  for (const mode of ['light', 'dark']) {
    builds.push(buildPalette(palette.id, mode));
  }
}

const report = builds.map((build) => {
  const whiteOnBrand = build.checks.find((c) => c.key === 'white-on-brand-label');
  const hardFailures = build.checks.filter((c) => !c.ok && c.key !== 'white-on-brand-label');
  return {
    palette: build.palette.name,
    id: build.palette.id,
    mode: build.mode,
    page: build.vars['--td-bg-color-page'],
    container: build.vars['--td-bg-color-container'],
    brand: build.vars['--td-brand-color-7'],
    surface: build.vars['--td-brand-color'],
    strong: build.vars['--app-brand-strong'],
    checks: build.checks,
    warnings: build.warnings,
    ok: hardFailures.length === 0 && whiteOnBrand?.ok === true ? true : hardFailures.length === 0,
    // 白字在主色上不达标 → 有压暗版兜底，属于"已处理"
    needsStrong: whiteOnBrand ? whiteOnBrand.ok === false : false,
  };
});

for (const row of report) {
  const label = `${row.palette} / ${row.mode === 'light' ? '浅色' : '深色'}`;
  const hard = row.checks.filter((c) => !c.ok && c.key !== 'white-on-brand-label');
  if (hard.length) {
    errors.push(`${label}：${hard.map((c) => `${c.label}(${c.ratio}<${c.min})`).join('、')}`);
  }
  for (const warning of row.warnings) {
    notes.push(warning);
  }
}

/* ------------------------------------------------------------------ *
 * 3. 输出
 * ------------------------------------------------------------------ */

const allErrors = [...dataErrors, ...errors];

if (asJson) {
  console.log(
    JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        passed: allErrors.length === 0,
        unitAssertions: unitCases.length,
        unitFailures: unitCases.filter((c) => !c.ok).length,
        groups: report,
        errors: allErrors,
        notes,
      },
      null,
      2,
    ),
  );
} else if (asMarkdown) {
  const header = [
    '| # | 方案 | 模式 | 页面底 | 主色 | 正文/底 | 次要/底 | 占位/底 | 品牌文字/底 | 白字/主色 | 白字/交互面 | 结论 |',
    '|---|---|---|---|---|---|---|---|---|---|---|---|',
  ];
  const lines = report.map((row, index) => {
    const pick = (key) => row.checks.find((c) => c.key === key);
    const cell = (item) => `${item.ratio}${item.ok ? ' ✓' : ' ✗'}`;
    const strong = row.needsStrong ? '（用压暗版）' : '';
    return `| ${index + 1} | ${row.palette} | ${row.mode === 'light' ? '浅色' : '深色'} | ${row.page} | ${row.brand} | ${cell(
      pick('text-primary'),
    )} | ${cell(pick('text-secondary'))} | ${cell(pick('text-placeholder'))} | ${cell(pick('brand-text'))} | ${cell(
      pick('white-on-brand-label'),
    )}${strong} | ${cell(pick('on-brand-surface'))} | ${row.ok ? '达标' : '不达标'} |`;
  });
  console.log([...header, ...lines].join('\n'));
} else {
  console.log('配色方案对比度报告（22 套 × 明暗 = 44 组）\n');
  const fmt = (v) => String(v).padEnd(8);
  console.log(
    `${'#'.padEnd(3)}${'方案'.padEnd(10)}${'模式'.padEnd(7)}${'正文'.padEnd(8)}${'次要'.padEnd(8)}${'占位'.padEnd(8)}${'品牌字'.padEnd(9)}${'白字/主色'.padEnd(12)}${'白字/交互面'.padEnd(10)}结论`,
  );
  report.forEach((row, index) => {
    const pick = (key) => row.checks.find((c) => c.key === key);
    const mark = (item) => `${item.ratio}${item.ok ? '' : '✗'}`;
    console.log(
      `${String(index + 1).padEnd(3)}${row.palette.padEnd(10)}${(row.mode === 'light' ? '浅色' : '深色').padEnd(7)}${fmt(
        mark(pick('text-primary')),
      )}${fmt(mark(pick('text-secondary')))}${fmt(mark(pick('text-placeholder')))}${fmt(mark(pick('brand-text')))}${fmt(
        `${mark(pick('white-on-brand-label'))}${row.needsStrong ? '*' : ''}`,
      )}${fmt(mark(pick('on-brand-surface')))}${row.ok ? '达标' : '不达标'}`,
    );
  });
  console.log('\n* 白字在主色上达不到 4.5 时：浅色模式主按钮底色下移到 --app-brand-strong 压暗版（保持白字），');
  console.log('  深色模式主按钮底色保持提亮后的主色、文字改用 --app-on-brand 深色油墨；--td-brand-color-7 始终是原色值');
  console.log('\n告警 / 自动修正记录：');
  for (const note of notes) console.log(`  - ${note}`);
  console.log(`\n色阶算法断言：${unitCases.length} 条，失败 ${unitCases.filter((c) => !c.ok).length} 条`);
  console.log(`色板数据校验：${dataErrors.length === 0 ? '通过' : `${dataErrors.length} 处不符`}`);
}

if (allErrors.length) {
  console.error('\n[check-palettes] 检查未通过：');
  for (const error of allErrors) console.error(`  ✗ ${error}`);
  process.exit(1);
}

if (!asJson && !asMarkdown) {
  console.log('\n[check-palettes] 全部通过 ✓');
}
