<template>
  <div ref="host" class="echart" :style="{ height: `${height}px`, width: '100%' }"></div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import * as echarts from 'echarts';
import { paletteChartColors, usePalette } from '@/composables/usePalette';

/** 图表 props */
interface Props {
  /** echarts option，如 { xAxis: {...}, series: [...] } */
  option: echarts.EChartsCoreOption;
  /** 容器高度（px），宽度自适应父容器 */
  height?: number;
}

const props = withDefaults(defineProps<Props>(), { height: 280 });

const host = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;

/** 是否已成功写入过 option：容器零尺寸时 init 出的空图，就绪后必须补 setOption */
let hasRendered = false;

/** 监听容器尺寸变化后重算画布，配合栅格布局的响应式换行 */
let observer: ResizeObserver | null = null;

/** 容器任一边为 0（标签页隐藏 / 栅格尚未布局 / v-if 未就绪）时初始化会得到 0×0 画布 */
function hasSize(): boolean {
  return !!host.value && host.value.clientWidth > 0 && host.value.clientHeight > 0;
}

function render(): void {
  if (!chart || !hasSize()) return;
  chart.setOption(withPaletteTheme(props.option), true);
  hasRendered = true;
}

/**
 * 给未显式指定 `color` 的图表补上当前配色的色板（`withPaletteTheme` 内部调用）。
 *
 * 这一步是「图表跟随配色」的挂点：业务页面只写数据结构，配色由配色引擎统一注入，
 * 所以 8 套方案 × 明暗切换时折线/柱状/饼图会一起换色，而页面代码零改动。
 * 调用方自己指定了 `color` 时保持原样（局部强调色优先）。
 * 用户未选择方案时 `paletteChartColors()` 返回空数组 → 这里返回 `undefined`，不覆盖任何颜色。
 */
function pickPaletteColors(option: echarts.EChartsCoreOption): string[] | undefined {
  if (!option || (option as { color?: unknown }).color !== undefined) {
    return undefined;
  }
  const colors = paletteChartColors();
  return colors.length ? colors : undefined;
}

/** 坐标轴/图例的文字与线条颜色 */
interface ChartTextColors {
  /** 主文字（标题） */
  primary: string;
  /** 次要文字（轴标签、图例） */
  secondary: string;
  /** 轴线与网格线 */
  stroke: string;
}

/** 读当前配色注入的 CSS 变量（图表文本色必须跟着配色走，否则深色模式下轴标签会糊掉） */
function readChartTextColors(): ChartTextColors {
  const style = getComputedStyle(document.documentElement);
  const read = (name: string, fallback: string): string => style.getPropertyValue(name).trim() || fallback;
  return {
    primary: read('--td-text-color-primary', '#1f2329'),
    secondary: read('--td-text-color-secondary', '#4b5160'),
    stroke: read('--td-component-stroke', '#e7e7e7'),
  };
}

/** 把配色里的文字/线条色合进单个坐标轴（调用方显式指定的值优先） */
function mergeAxisStyle(axis: unknown, colors: ChartTextColors): unknown {
  const source = (axis ?? {}) as Record<string, unknown>;
  const axisLabel = (source.axisLabel ?? {}) as Record<string, unknown>;
  const axisLine = (source.axisLine ?? {}) as Record<string, unknown>;
  const axisLineStyle = (axisLine.lineStyle ?? {}) as Record<string, unknown>;
  const splitLine = (source.splitLine ?? {}) as Record<string, unknown>;
  const splitLineStyle = (splitLine.lineStyle ?? {}) as Record<string, unknown>;
  return {
    ...source,
    axisLabel: { color: colors.secondary, ...axisLabel },
    axisLine: { ...axisLine, lineStyle: { color: colors.stroke, ...axisLineStyle } },
    splitLine: { ...splitLine, lineStyle: { color: colors.stroke, ...splitLineStyle } },
  };
}

/**
 * 图表整体换肤：色板 + 文字色 + 轴线色。
 *
 * 调用方已经写死的字段一律保留（例如"成功/失败"两条曲线用语义色），只补缺省值，
 * 所以业务页面不需要为新配色改任何一行。
 */
function withPaletteTheme(option: echarts.EChartsCoreOption): echarts.EChartsCoreOption {
  if (!option) {
    return option;
  }
  // 「跟随默认」（用户未选择任何方案）→ 完全不改动 option，
  // 图表保持升级前的配色（含坐标轴/图例文字色），这是"界面零变化"的一部分。
  if (!paletteId.value) {
    return option;
  }
  const colors = readChartTextColors();
  const source = option as Record<string, unknown>;
  const next: Record<string, unknown> = { ...source };
  const paletteColors = pickPaletteColors(option);
  if (paletteColors) {
    next.color = paletteColors;
  }
  if (next.textStyle === undefined) {
    next.textStyle = { color: colors.primary };
  }
  for (const key of ['xAxis', 'yAxis'] as const) {
    const axis = next[key];
    if (axis) {
      next[key] = Array.isArray(axis) ? axis.map((item) => mergeAxisStyle(item, colors)) : mergeAxisStyle(axis, colors);
    }
  }
  if (next.legend) {
    const legend = next.legend as Record<string, unknown>;
    next.legend = { ...legend, textStyle: { color: colors.secondary, ...(legend.textStyle as object) } };
  }
  return next as echarts.EChartsCoreOption;
}

function resize(): void {
  if (!chart) return;
  // 从未渲染过（挂载时容器零尺寸）→ 尺寸就绪后先补渲染，而不是 resize 空图
  if (!hasRendered) {
    render();
    return;
  }
  if (hasSize()) chart.resize();
}

onMounted(() => {
  if (!host.value) return;
  // 等一拍再 init：父级 loading / 栅格布局可能在挂载瞬间还没给出最终尺寸
  void nextTick(() => {
    if (!host.value) return;
    chart = echarts.init(host.value);
    render();
    // 侧边栏折叠、窗口缩放、栅格换行都会改变容器宽度
    if (typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(() => resize());
      observer.observe(host.value);
    }
    window.addEventListener('resize', resize);
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', resize);
  observer?.disconnect();
  observer = null;
  chart?.dispose();
  chart = null;
});

// nextTick：等父组件完成本次渲染、容器拿到最终尺寸后再写入
watch(() => props.option, () => void nextTick(render), { deep: true });

/**
 * 配色 / 明暗切换后重绘。
 *
 * option 本身没变，变的是 CSS 变量与色板，所以必须单独订阅配色版本号，
 * 否则切换配色后图表会停留在旧色（"图表不跟随"的典型原因）。
 */
const { paletteId, version: paletteVersion } = usePalette();
watch(paletteVersion, () => void nextTick(render));
</script>

<style scoped lang="less">
.echart {
  width: 100%;
}
</style>
