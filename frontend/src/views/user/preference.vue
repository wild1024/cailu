<template>
  <div class="preference-page">
    <!-- 顶部说明 -->
    <div class="preference-hero">
      <div class="preference-hero__text">
        <h2 class="preference-hero__title">外观设置</h2>
        <p class="preference-hero__desc">
          选择顺眼的配色方案与明暗模式，点击即生效并自动记住（仅影响当前浏览器）。
          共 {{ paletteCount }} 套方案 × 明暗 2 种模式，全部经过 WCAG AA 对比度校验。
          <strong>默认不应用任何方案</strong>：不选就与升级前的界面完全一致（原青蓝色系）。
        </p>
      </div>
      <t-button variant="outline" @click="resetPalette">
        <template #icon><t-icon name="rollback" /></template>
        跟随默认（保持原主题）
      </t-button>
    </div>

    <!-- 明暗模式 -->
    <t-card class="preference-card" :bordered="false" title="明暗模式">
      <template #actions>
        <span class="preference-card__hint">当前：{{ mode === 'dark' ? '深色' : '浅色' }}</span>
      </template>
      <div class="mode-switch">
        <button
          v-for="item in modeOptions"
          :key="item.value"
          type="button"
          class="mode-switch__item"
          :class="{ 'is-active': mode === item.value }"
          :data-mode-option="item.value"
          @click="setMode(item.value)"
        >
          <t-icon :name="item.icon" class="mode-switch__icon" />
          <span class="mode-switch__label">{{ item.label }}</span>
          <span class="mode-switch__desc">{{ item.desc }}</span>
          <t-icon v-if="mode === item.value" name="check-circle-filled" class="mode-switch__check" />
        </button>
      </div>
    </t-card>

    <!-- 跟随默认 + 8 套配色 -->
    <t-card class="preference-card" :bordered="false" title="配色方案">
      <template #actions>
        <span class="preference-card__hint">5 个基准色：主色 / 浅色 / 交融色（页面底） / 渐染色（边框） / 强调色</span>
      </template>
      <t-row :gutter="[16, 16]">
        <!-- 第一项永远是「跟随默认」：它不是第 9 套方案，而是「不选择」这一状态 -->
        <t-col :xs="12" :sm="12" :md="6" :xl="4">
          <button
            type="button"
            class="palette-card palette-card--default"
            :class="{ 'is-active': isDefault }"
            data-palette-option="default"
            :aria-pressed="isDefault"
            @click="resetPalette()"
          >
            <span class="palette-card__swatches palette-card__swatches--default" />
            <span class="palette-card__body">
              <span class="palette-card__name">跟随默认</span>
              <span class="palette-card__note">保持原主题，不应用任何配色方案</span>
            </span>
            <span v-if="isDefault" class="palette-card__check">
              <t-icon name="check" />
            </span>
          </button>
        </t-col>
        <t-col v-for="item in paletteCards" :key="item.id" :xs="12" :sm="12" :md="6" :xl="4">
          <t-tooltip :content="item.note" placement="bottom" show-arrow>
            <button
              type="button"
              class="palette-card"
              :class="{ 'is-active': item.active }"
              :data-palette-option="item.id"
              :aria-pressed="item.active"
              @click="setPalette(item.id)"
            >
              <!-- 色卡：5 个基准色横排 -->
              <span class="palette-card__swatches">
                <i
                  v-for="(color, index) in item.colors"
                  :key="`${item.id}-${index}`"
                  class="palette-card__swatch"
                  :style="{ backgroundColor: color }"
                  :title="item.colorLabels[index]"
                />
              </span>

              <span class="palette-card__body">
                <span class="palette-card__name">{{ item.name }}</span>
                <span class="palette-card__note">{{ item.shortNote }}</span>
              </span>

              <span v-if="item.active" class="palette-card__check">
                <t-icon name="check" />
              </span>

              <span v-if="item.needsStrong" class="palette-card__badge">白字自动压暗</span>
            </button>
          </t-tooltip>
        </t-col>
      </t-row>
      <p class="preference-note">
        主色已按规定色值原样使用（<code>--td-brand-color-7</code>）；若白字在主色上达不到 4.5:1，
        浅色模式会把主按钮底色自动下移到压暗版 <code>--app-brand-strong</code>，
        深色模式则把按钮文字换成 <code>--app-on-brand</code> 深色油墨 —— 主色本身始终不变
        （标「白字自动压暗」的方案即属于这一类）。
      </p>
    </t-card>

    <!-- 实时预览 -->
    <t-card class="preference-card" :bordered="false" title="实时预览">
      <template #actions>
        <span class="preference-card__hint">当前：{{ isDefault ? '跟随默认' : currentPalette?.name }} · {{ mode === 'dark' ? '深色' : '浅色' }}</span>
      </template>

      <t-row :gutter="[24, 24]">
        <t-col :xs="12" :md="12" :xl="7">
          <div class="preview-block">
            <div class="preview-row">
              <t-button theme="primary">主要按钮</t-button>
              <t-button theme="primary" variant="outline">次要按钮</t-button>
              <t-button variant="text">文字按钮</t-button>
              <t-button theme="primary" disabled>禁用态</t-button>
            </div>

            <div class="preview-row">
              <t-tag theme="primary" variant="light">标签</t-tag>
              <t-tag theme="success" variant="light">成功</t-tag>
              <t-tag theme="warning" variant="light">警告</t-tag>
              <t-tag theme="danger" variant="light">危险</t-tag>
              <t-tag theme="primary">实心标签</t-tag>
            </div>

            <div class="preview-text">
              <p class="preview-text__primary">正文示例：这是一段用于检查可读性的正文文字，对比度 ≥ 4.5。</p>
              <p class="preview-text__secondary">次要文字示例：说明性文案、辅助信息走这一级，同样 ≥ 4.5。</p>
              <p class="preview-text__placeholder">占位文字示例：请输入关键字（≥ 3.0）</p>
              <p class="preview-text__link">品牌文字示例：<span class="preview-link">查看详情 →</span></p>
            </div>

            <div class="preview-row">
              <t-input placeholder="输入框占位文字" style="width: 200px" />
              <t-switch :value="true" />
              <t-radio-group :value="'a'" variant="default-filled">
                <t-radio-button value="a">选项一</t-radio-button>
                <t-radio-button value="b">选项二</t-radio-button>
              </t-radio-group>
            </div>

            <table class="preview-table">
              <thead>
                <tr>
                  <th>指标</th>
                  <th>数值</th>
                  <th>状态</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in previewRows" :key="row.name">
                  <td>{{ row.name }}</td>
                  <td class="preview-table__num">{{ row.value }}</td>
                  <td>
                    <t-tag :theme="row.theme" variant="light" size="small">{{ row.status }}</t-tag>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </t-col>

        <t-col :xs="12" :md="12" :xl="5">
          <div class="preview-block preview-block--chart">
            <p class="preview-block__title">图表配色跟随</p>
            <EChart :option="previewChartOption" :height="220" />
          </div>
        </t-col>
      </t-row>
    </t-card>

    <!-- 对比度报告 -->
    <t-card class="preference-card" :bordered="false" title="对比度报告（16 组）">
      <template #actions>
        <span class="preference-card__hint">
          <t-tag theme="success" variant="light" size="small">全部达标</t-tag>
          AA 门槛：正文 4.5 / 次要 4.5 / 占位 3.0 / 品牌文字 4.5
        </span>
      </template>
      <div class="contrast-table-wrap">
        <table class="contrast-table">
          <thead>
            <tr>
              <th>方案</th>
              <th>模式</th>
              <th>页面底</th>
              <th>正文/底</th>
              <th>次要/底</th>
              <th>占位/底</th>
              <th>品牌字/底</th>
              <th>白字/主色</th>
              <th>主色上文字/交互面</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in auditRows"
              :key="`${row.id}-${row.mode}`"
              :class="{ 'is-current': row.id === paletteId && row.mode === mode }"
            >
              <td>{{ row.name }}</td>
              <td>{{ row.mode === 'dark' ? '深色' : '浅色' }}</td>
              <td>
                <span class="contrast-table__chip" :style="{ backgroundColor: row.page }" />
                <code>{{ row.page }}</code>
              </td>
              <td :class="okClass(row.textPrimary)">{{ row.textPrimary }}</td>
              <td :class="okClass(row.textSecondary)">{{ row.textSecondary }}</td>
              <td :class="okClass(row.placeholder)">{{ row.placeholder }}</td>
              <td :class="okClass(row.brandText)">{{ row.brandText }}</td>
              <td :class="okClass(row.whiteOnBrand)">
                {{ row.whiteOnBrand
                }}<span v-if="row.needsStrong" class="contrast-table__flag">{{
                  row.mode === 'light' ? '压暗版' : '深色字'
                }}</span>
              </td>
              <td :class="okClass(row.surfaceText)">{{ row.surfaceText }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="preference-note">
        数值来自 <code>scripts/check-palettes.mjs</code> 同一份推导代码（<code>pnpm check:palettes</code> 可随时重跑）。
        「白字/主色」标「压暗版」表示该组合下主按钮底色自动下移到压暗版，保证白字可读。
      </p>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { EChartsCoreOption } from 'echarts';
import EChart from '@/components/EChart.vue';
import { usePalette } from '@/composables/usePalette';
import { PALETTES, buildPalette, type ColorMode } from '@/style/palettes';

const {
  palettes,
  paletteId,
  currentPalette,
  isDefault,
  mode,
  setPalette,
  setMode,
  resetPalette,
  buildsByPalette,
} = usePalette();

/** 可选方案数量（8 套；「跟随默认」不计入） */
const paletteCount = palettes.length;

/** 明暗模式选项（卡片式单选） */
const modeOptions: { value: ColorMode; label: string; desc: string; icon: string }[] = [
  { value: 'light', label: '浅色', desc: '交融色作页面底，深色文字', icon: 'sunny' },
  { value: 'dark', label: '深色', desc: '交融色压暗作底，浅色文字', icon: 'moon' },
];

/** 色卡数据：5 个基准色 + 是否需压暗版兜底 */
const paletteCards = computed(() =>
  PALETTES.map((item) => {
    const build = buildsByPalette.value[item.id];
    const whiteOnBrand = build.checks.find((check) => check.key === 'white-on-brand-label');
    const noteShort = item.note.split('——')[0] ?? item.note;
    return {
      id: item.id,
      name: item.name,
      note: item.note,
      shortNote: noteShort.trim(),
      colors: [item.brand, item.brandLight, item.merge, item.border, item.accent],
      colorLabels: ['主色 brand-7', '浅色 brand-5', '交融色（页面底）', '渐染色（边框）', '强调色 accent'],
      active: paletteId.value === item.id,
      needsStrong: whiteOnBrand ? !whiteOnBrand.ok : false,
    };
  }),
);

/** 预览用表格数据 */
const previewRows = [
  { name: '今日活跃用户', value: '1,284', status: '正常', theme: 'success' as const },
  { name: '平均登录耗时', value: '186 ms', status: '关注', theme: 'warning' as const },
  { name: '失败率', value: '2.4%', status: '偏高', theme: 'danger' as const },
];

/** 预览图表：不指定 color，交给 EChart 注入当前配色 */
const previewChartOption = computed<EChartsCoreOption>(() => ({
  grid: { left: 40, right: 16, top: 24, bottom: 28 },
  tooltip: { trigger: 'axis' },
  xAxis: { type: 'category', data: ['周一', '周二', '周三', '周四', '周五', '周六', '周日'] },
  yAxis: { type: 'value' },
  series: [
    { name: '访问量', type: 'bar', data: [120, 200, 150, 80, 170, 220, 190], barWidth: 14 },
    { name: '活跃用户', type: 'line', smooth: true, data: [60, 120, 90, 50, 110, 140, 130] },
  ],
}));

/** 16 组（8 套 × 明暗）对比度数据（与检查脚本同源） */
const auditRows = computed(() => {
  const rows = PALETTES.flatMap((item) =>
    (['light', 'dark'] as ColorMode[]).map((itemMode) => {
      const build = buildPalette(item.id, itemMode);
      const pick = (key: string): number => build.checks.find((check) => check.key === key)?.ratio ?? 0;
      const whiteOnBrand = build.checks.find((check) => check.key === 'white-on-brand-label');
      const surfaceText = build.checks.find((check) => check.key === 'on-brand-surface');
      return {
        id: item.id,
        name: item.name,
        mode: itemMode,
        page: build.vars['--td-bg-color-page'],
        textPrimary: pick('text-primary'),
        textSecondary: pick('text-secondary'),
        placeholder: pick('text-placeholder'),
        brandText: pick('brand-text'),
        whiteOnBrand: whiteOnBrand ? whiteOnBrand.ratio : 0,
        surfaceText: surfaceText ? surfaceText.ratio : 0,
        needsStrong: whiteOnBrand ? !whiteOnBrand.ok : false,
      };
    }),
  );
  return rows;
});

/** 对比度数值着色：达标绿、不达标红 */
function okClass(value: number): string {
  return value >= 4.5 ? 'is-pass' : value >= 3 ? 'is-warn' : 'is-fail';
}
</script>

<style scoped lang="less">
.preference-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.preference-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 4px 4px 0;

  &__title {
    margin: 0 0 6px;
    font-size: 20px;
    font-weight: 600;
    color: var(--td-text-color-primary);
  }

  &__desc {
    max-width: 720px;
    margin: 0;
    font-size: 13px;
    line-height: 1.7;
    color: var(--td-text-color-secondary);
  }
}

.preference-card {
  border-radius: 12px;
  background-color: var(--td-bg-color-container);

  &__hint {
    font-size: 12px;
    color: var(--td-text-color-placeholder);
  }
}

.preference-note {
  margin: 12px 0 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--td-text-color-secondary);

  code {
    padding: 1px 4px;
    font-size: 12px;
    color: var(--td-text-color-primary);
    background-color: var(--td-bg-color-secondarycontainer);
    border-radius: 4px;
  }
}

/* ---------- 明暗模式卡片 ---------- */
.mode-switch {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;

  &__item {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 220px;
    padding: 14px 18px;
    text-align: left;
    cursor: pointer;
    background-color: var(--td-bg-color-container);
    border: 1px solid var(--td-component-border);
    border-radius: 10px;
    transition:
      border-color 0.18s ease,
      background-color 0.18s ease,
      transform 0.18s ease;

    &:hover {
      border-color: var(--td-brand-color-5);
      transform: translateY(-1px);
    }

    &.is-active {
      background-color: var(--td-brand-color-1);
      border-color: var(--td-brand-color);
      box-shadow: 0 0 0 1px var(--td-brand-color) inset;
    }
  }

  &__icon {
    font-size: 20px;
    color: var(--td-brand-color);
  }

  &__label {
    font-size: 15px;
    font-weight: 600;
    color: var(--td-text-color-primary);
  }

  &__desc {
    font-size: 12px;
    color: var(--td-text-color-secondary);
  }

  &__check {
    position: absolute;
    top: 12px;
    right: 12px;
    font-size: 18px;
    color: var(--td-brand-color);
  }
}

/* ---------- 色卡 ---------- */
.palette-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  padding: 12px;
  text-align: left;
  cursor: pointer;
  background-color: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: 12px;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;

  &:hover {
    border-color: var(--td-brand-color-5);
    box-shadow: 0 6px 18px rgb(0 0 0 / 8%);
    transform: translateY(-2px);
  }

  &.is-active {
    border-color: var(--td-brand-color);
    box-shadow: 0 0 0 2px var(--td-brand-color) inset;
  }

  &__swatches {
    display: flex;
    height: 46px;
    overflow: hidden;
    border-radius: 8px;
  }

  &__swatch {
    flex: 1;
    transition: flex 0.2s ease;

    &:hover {
      flex: 1.6;
    }
  }

  // 「跟随默认」色卡没有 5 个基准色，用当前（原）主题的横幅渐变直观表示"不加任何方案"
  &--default &__swatches,
  &__swatches--default {
    background: var(--app-gradient-brand);
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--td-text-color-primary);
  }

  &__note {
    font-size: 12px;
    color: var(--td-text-color-secondary);
  }

  &__check {
    position: absolute;
    top: 8px;
    right: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    font-size: 13px;
    color: #fff;
    background-color: var(--td-brand-color);
    border-radius: 50%;
    box-shadow: 0 2px 6px rgb(0 0 0 / 18%);
  }

  &__badge {
    align-self: flex-start;
    padding: 0 6px;
    font-size: 11px;
    line-height: 18px;
    color: var(--td-text-color-secondary);
    background-color: var(--td-bg-color-secondarycontainer);
    border-radius: 4px;
  }
}

/* ---------- 实时预览 ---------- */
.preview-block {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;

  &--chart {
    padding: 12px;
    background-color: var(--td-bg-color-container);
    border: 1px solid var(--td-component-stroke);
    border-radius: 10px;
  }

  &__title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--td-text-color-secondary);
  }
}

.preview-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.preview-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background-color: var(--td-bg-color-page);
  border: 1px solid var(--td-component-stroke);
  border-radius: 10px;

  p {
    margin: 0;
    font-size: 13px;
  }

  &__primary {
    color: var(--td-text-color-primary);
  }

  &__secondary {
    color: var(--td-text-color-secondary);
  }

  &__placeholder {
    color: var(--td-text-color-placeholder);
  }

  &__link {
    color: var(--td-text-color-primary);
  }
}

.preview-link {
  color: var(--td-text-color-brand);
  cursor: pointer;

  &:hover {
    text-decoration: underline;
  }
}

.preview-table {
  width: 100%;
  font-size: 13px;
  border-collapse: collapse;

  th,
  td {
    padding: 8px 10px;
    text-align: left;
    border-bottom: 1px solid var(--td-component-stroke);
  }

  th {
    font-weight: 600;
    color: var(--td-text-color-secondary);
    background-color: var(--td-bg-color-secondarycontainer);
  }

  td {
    color: var(--td-text-color-primary);
  }

  &__num {
    font-variant-numeric: tabular-nums;
  }
}

/* ---------- 对比度报告 ---------- */
.contrast-table-wrap {
  overflow-x: auto;
}

.contrast-table {
  width: 100%;
  font-size: 12px;
  border-collapse: collapse;

  th,
  td {
    padding: 7px 10px;
    text-align: left;
    white-space: nowrap;
    border-bottom: 1px solid var(--td-component-stroke);
  }

  th {
    font-weight: 600;
    color: var(--td-text-color-secondary);
    background-color: var(--td-bg-color-secondarycontainer);
  }

  td {
    color: var(--td-text-color-primary);
  }

  tbody tr.is-current {
    // 当前组合只做左侧标示，不铺底色：铺品牌色底会让表格里的红/绿数值失去对比度
    box-shadow: inset 3px 0 0 var(--td-brand-color);
  }

  &__chip {
    display: inline-block;
    width: 12px;
    height: 12px;
    margin-right: 6px;
    vertical-align: -2px;
    border: 1px solid var(--td-component-border);
    border-radius: 3px;
  }

  &__flag {
    margin-left: 4px;
    padding: 0 4px;
    font-size: 11px;
    color: var(--td-text-color-secondary);
    background-color: var(--td-bg-color-secondarycontainer);
    border-radius: 3px;
  }

  .is-pass {
    color: var(--td-success-color);
  }

  .is-warn {
    color: var(--td-warning-color);
  }

  .is-fail {
    color: var(--td-error-color);
    font-weight: 600;
  }
}
</style>
