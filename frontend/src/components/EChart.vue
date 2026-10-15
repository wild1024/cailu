<template>
  <div ref="host" class="echart" :style="{ height: `${height}px`, width: '100%' }"></div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import * as echarts from 'echarts';

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
  chart.setOption(props.option, true);
  hasRendered = true;
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
</script>

<style scoped lang="less">
.echart {
  width: 100%;
}
</style>
