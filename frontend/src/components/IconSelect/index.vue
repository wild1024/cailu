<template>
  <t-popup
    v-model:visible="popupVisible"
    trigger="click"
    placement="bottom-left"
    :overlay-inner-style="{ width: '440px', maxWidth: '92vw', padding: 0 }"
    destroy-on-close
    @visible-change="onVisibleChange"
  >
    <!-- 触发器：图标输入框，可直接输入图标名，也可点击从下拉中选 -->
    <t-input
      v-model="innerValue"
      :placeholder="placeholder"
      :disabled="disabled"
      clearable
      @click="onTriggerClick"
      @enter="onTriggerClick"
    >
      <template #prefixIcon>
        <t-icon v-if="innerValue" :name="innerValue" size="18px" />
      </template>
      <template #suffixIcon>
        <t-icon name="chevron-down" size="16px" class="icon-select__caret" />
      </template>
    </t-input>

    <template #content>
      <div class="icon-select__inner">
        <div class="icon-select__search">
          <t-input v-model="keyword" clearable size="small" placeholder="输入名称搜索，如 user / folder">
            <template #prefixIcon>
              <t-icon name="search" size="16px" />
            </template>
          </t-input>
        </div>

        <div ref="bodyRef" class="icon-select__body" @scroll.passive="onBodyScroll">
          <div v-if="icons.length" class="icon-select__grid">
            <button
              v-for="name in visibleIcons"
              :key="name"
              type="button"
              class="icon-select__item"
              :class="{ 'icon-select__item--active': name === innerValue }"
              :title="name"
              @click="pick(name)"
            >
              <t-icon :name="name" size="22px" />
              <span class="icon-select__label">{{ name }}</span>
            </button>
          </div>
          <div v-else class="icon-select__empty">
            <t-icon name="info-circle" size="32px" />
            <span>没有匹配的图标</span>
          </div>
        </div>

        <div class="icon-select__footer">
          <span class="icon-select__count">共 {{ icons.length }} 个图标{{ icons.length > visibleIcons.length ? `，已加载 ${visibleIcons.length}` : '' }}</span>
          <t-space size="4px">
            <t-button size="small" variant="text" theme="danger" :disabled="!innerValue" @click="clearAll">
              清空
            </t-button>
            <t-button size="small" variant="outline" @click="popupVisible = false">关闭</t-button>
          </t-space>
        </div>
      </div>
    </template>
  </t-popup>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { filterIconNames } from './icons';

const props = withDefaults(
  defineProps<{
    /** 当前图标名 */
    modelValue?: string;
    /** 是否禁用 */
    disabled?: boolean;
    /** 占位提示 */
    placeholder?: string;
  }>(),
  { modelValue: '', disabled: false, placeholder: '请选择或输入图标名，如 user' },
);

const emit = defineEmits<{
  /** 值变化，清空时回传空串 */
  (e: 'update:modelValue', value: string): void;
}>();

/** 每次滚到底部额外渲染的图标数量，避免一次性渲染上千个 SVG */
const BATCH = 240;
const INITIAL = BATCH;

const innerValue = ref(props.modelValue);
const popupVisible = ref(false);
const keyword = ref('');
const rendered = ref(INITIAL);
const bodyRef = ref<HTMLElement | null>(null);

watch(
  () => props.modelValue,
  (value) => {
    innerValue.value = value ?? '';
  },
);

watch(innerValue, (value) => {
  emit('update:modelValue', value ?? '');
  // 面板展开时在触发器里直接输入，网格需同步过滤
  if (popupVisible.value) {
    keyword.value = value ?? '';
    rendered.value = INITIAL;
  }
});

/** 当前关键字下的图标全集 */
const icons = computed(() => filterIconNames(keyword.value));

/** 已渲染的图标（增量加载） */
const visibleIcons = computed(() => icons.value.slice(0, rendered.value));

/** 滚动触底时追加渲染，保持长列表流畅 */
function onBodyScroll(event: Event): void {
  const el = event.target as HTMLElement;
  const reachedBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 80;
  if (reachedBottom && rendered.value < icons.value.length) {
    rendered.value += BATCH;
  }
}

/** 选中图标：写回表单值并收起面板 */
function pick(name: string): void {
  innerValue.value = name;
  emit('update:modelValue', name);
  popupVisible.value = false;
}

/** 清空：同时重置面板搜索条件 */
function clearAll(): void {
  innerValue.value = '';
  emit('update:modelValue', '');
  keyword.value = '';
  rendered.value = INITIAL;
  popupVisible.value = false;
}

/** 点击输入框展开面板 */
function onTriggerClick(): void {
  popupVisible.value = true;
}

/**
 * 面板开关控制。
 * popup 的 trigger="click" 在点击触发器时会自行置为 false，
 * 这里再置回 true，避免"点输入框一下面板就收起"；
 * 点击浮层外其它区域（document）则正常收起。
 */
function onVisibleChange(visible: boolean, context: { trigger?: string }): void {
  if (visible) {
    // 每次展开都以当前值作为搜索关键字，选中项能在网格里直接定位到
    keyword.value = innerValue.value ?? '';
    rendered.value = INITIAL;
    void nextTick(() => {
      if (bodyRef.value) bodyRef.value.scrollTop = 0;
    });
    return;
  }
  if (context.trigger === 'trigger-element-click') {
    popupVisible.value = true;
  }
}
</script>

<style lang="less" scoped>
.icon-select {
  width: 100%;

  .icon-select__caret {
    color: var(--td-text-color-placeholder);
    transition: transform 0.2s;
  }
}

.icon-select__inner {
  display: flex;
  flex-direction: column;
  padding: 8px;
}

.icon-select__search {
  padding: 0 4px 8px;
}

.icon-select__body {
  min-height: 200px;
  max-height: 288px;
  overflow-y: auto;
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  background-color: var(--td-bg-color-container);
}

.icon-select__grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 4px;
  padding: 8px;
}

.icon-select__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 56px;
  padding: 4px;
  overflow: hidden;
  border: 1px solid transparent;
  border-radius: var(--td-radius-small);
  background: transparent;
  cursor: pointer;
  transition: background-color 0.2s, border-color 0.2s;

  &:hover {
    border-color: var(--td-brand-color);
    background-color: var(--td-brand-color-light);

    .icon-select__label {
      color: var(--td-brand-color);
    }
  }

  &--active {
    border-color: var(--td-brand-color);
    background-color: var(--td-brand-color-light);

    .icon-select__label {
      color: var(--td-brand-color);
      font-weight: 600;
    }
  }
}

.icon-select__label {
  max-width: 100%;
  overflow: hidden;
  color: var(--td-text-color-secondary);
  font-size: 12px;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-select__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 200px;
  color: var(--td-text-color-placeholder);
  font-size: 13px;
}

.icon-select__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px 0;
}

.icon-select__count {
  color: var(--td-text-color-secondary);
  font-size: 12px;
}
</style>
