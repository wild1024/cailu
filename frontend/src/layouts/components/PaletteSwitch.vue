<template>
  <t-popup
    v-model:visible="visible"
    trigger="click"
    placement="bottom-right"
    :show-arrow="false"
    overlay-class-name="palette-switch__popup"
  >
    <t-tooltip content="配色方案" placement="bottom">
      <t-button
        class="palette-switch__trigger"
        variant="text"
        shape="square"
        data-testid="palette-switch-trigger"
      >
        <t-icon name="palette" />
      </t-button>
    </t-tooltip>

    <template #content>
      <div class="palette-panel">
        <div class="palette-panel__head">
          <span class="palette-panel__title">配色方案</span>
          <span class="palette-panel__sub">当前：{{ isDefault ? '跟随默认' : currentPalette?.name }} · {{ mode === 'dark' ? '深色' : '浅色' }}</span>
        </div>

        <!-- 第一项永远是「跟随默认」：不应用任何方案，回到原主题 -->
        <button
          type="button"
          class="palette-panel__default"
          :class="{ 'is-active': isDefault }"
          data-palette-option="quick-default"
          @click="resetPalette()"
        >
          <t-icon name="rollback" class="palette-panel__default-icon" />
          <span class="palette-panel__default-text">
            <span class="palette-panel__default-label">跟随默认（保持原主题）</span>
            <span class="palette-panel__default-hint">不应用任何配色方案，界面与升级前一致</span>
          </span>
          <t-icon v-if="isDefault" name="check" class="palette-panel__check" />
        </button>

        <!-- 明暗模式：与配色正交，切换互不影响 -->
        <div class="palette-panel__modes">
          <button
            v-for="item in modeOptions"
            :key="item.value"
            type="button"
            class="palette-panel__mode"
            :class="{ 'is-active': mode === item.value }"
            :data-mode-option="`quick-${item.value}`"
            @click="setMode(item.value)"
          >
            <t-icon :name="item.icon" />
            {{ item.label }}
          </button>
        </div>

        <div class="palette-panel__grid">
          <button
            v-for="item in cards"
            :key="item.id"
            type="button"
            class="palette-panel__item"
            :class="{ 'is-active': item.active }"
            :title="item.note"
            :data-palette-option="`quick-${item.id}`"
            @click="setPalette(item.id)"
          >
            <span class="palette-panel__swatches">
              <i
                v-for="(color, index) in item.colors"
                :key="`${item.id}-${index}`"
                :style="{ backgroundColor: color }"
              />
            </span>
            <span class="palette-panel__name">
              {{ item.name }}
              <t-icon v-if="item.active" name="check" class="palette-panel__check" />
            </span>
          </button>
        </div>

        <button type="button" class="palette-panel__more" @click="goPreference">
          前往「外观设置」查看对比度报告
          <t-icon name="chevron-right" />
        </button>
      </div>
    </template>
  </t-popup>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { PALETTES, type ColorMode } from '@/style/palettes';
import { usePalette } from '@/composables/usePalette';

/** 面板显隐（点击触发，点外部自动关闭） */
const visible = ref(false);

const router = useRouter();
const { paletteId, currentPalette, isDefault, mode, setPalette, setMode, resetPalette } = usePalette();

/** 明暗模式选项 */
const modeOptions: { value: ColorMode; label: string; icon: string }[] = [
  { value: 'light', label: '浅色', icon: 'sunny' },
  { value: 'dark', label: '深色', icon: 'moon' },
];

/** 面板内的 8 个迷你色卡（不含「跟随默认」那一项） */
const cards = computed(() =>
  PALETTES.map((item) => ({
    id: item.id,
    name: item.name,
    note: item.note,
    colors: [item.brand, item.brandLight, item.merge, item.border, item.accent],
    active: paletteId.value === item.id,
  })),
);

/** 跳到设置页的「外观」区块 */
function goPreference(): void {
  visible.value = false;
  void router.push('/user/preference');
}
</script>

<style scoped lang="less">
.palette-switch {
  &__trigger {
    flex-shrink: 0;
    color: var(--td-text-color-secondary);
  }
}
</style>

<!-- 弹层挂载到 body，需要非 scoped 样式 -->
<style lang="less">
.palette-switch__popup {
  .t-popup__content {
    padding: 0;
    border-radius: 12px;
    box-shadow: 0 8px 28px rgb(0 0 0 / 14%);
    background-color: var(--td-bg-color-container);
  }
}

.palette-panel {
  width: 328px;
  padding: 14px;

  &__head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  &__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--td-text-color-primary);
  }

  &__sub {
    font-size: 12px;
    color: var(--td-text-color-placeholder);
  }

  &__modes {
    display: flex;
    gap: 8px;
    padding: 4px;
    margin-bottom: 12px;
    background-color: var(--td-bg-color-secondarycontainer);
    border-radius: 8px;
  }

  &__mode {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 6px 0;
    font-size: 13px;
    color: var(--td-text-color-secondary);
    cursor: pointer;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    transition:
      background-color 0.18s ease,
      color 0.18s ease;

    &:hover {
      color: var(--td-text-color-primary);
    }

    &.is-active {
      color: var(--td-text-color-primary);
      background-color: var(--td-bg-color-container);
      border-color: var(--td-component-border);
      box-shadow: 0 1px 3px rgb(0 0 0 / 6%);
    }
  }

  &__grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }

  // 「跟随默认」：横跨整行，视觉上独立于 8 个色卡（它不是第 9 套方案）
  &__default {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    margin-bottom: 10px;
    text-align: left;
    cursor: pointer;
    background-color: var(--td-bg-color-container);
    border: 1px solid var(--td-component-stroke);
    border-radius: 8px;
    transition: border-color 0.18s ease;

    &:hover {
      border-color: var(--td-brand-color-5);
    }

    &.is-active {
      border-color: var(--td-brand-color);
      box-shadow: 0 0 0 1px var(--td-brand-color) inset;
    }
  }

  &__default-icon {
    flex-shrink: 0;
    color: var(--td-text-color-secondary);
  }

  &__default-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  &__default-label {
    font-size: 13px;
    color: var(--td-text-color-primary);
  }

  &__default-hint {
    font-size: 11px;
    color: var(--td-text-color-placeholder);
  }

  &__item {
    padding: 6px;
    text-align: left;
    cursor: pointer;
    background-color: var(--td-bg-color-container);
    border: 1px solid var(--td-component-stroke);
    border-radius: 8px;
    transition:
      border-color 0.18s ease,
      transform 0.18s ease;

    &:hover {
      border-color: var(--td-brand-color-5);
      transform: translateY(-1px);
    }

    &.is-active {
      border-color: var(--td-brand-color);
      box-shadow: 0 0 0 1px var(--td-brand-color) inset;
    }
  }

  &__swatches {
    display: flex;
    height: 22px;
    overflow: hidden;
    border-radius: 4px;

    i {
      flex: 1;
    }
  }

  &__name {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 5px;
    font-size: 12px;
    color: var(--td-text-color-primary);
  }

  &__check {
    font-size: 12px;
    color: var(--td-brand-color);
  }

  &__more {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    width: 100%;
    padding: 8px 0 2px;
    margin-top: 10px;
    font-size: 12px;
    color: var(--td-text-color-secondary);
    cursor: pointer;
    background: transparent;
    border: none;
    border-top: 1px solid var(--td-component-stroke);

    &:hover {
      color: var(--td-brand-color);
    }
  }
}
</style>
