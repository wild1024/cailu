<template>
  <t-select
    :value="modelValue || undefined"
    :options="options"
    :clearable="clearable"
    :disabled="disabled"
    :placeholder="placeholder"
    @change="onChange"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue';
import { useDictStore } from '@/stores/dict';

const props = withDefaults(
  defineProps<{
    /** 字典编码 */
    dictCode: string;
    /** 当前值（字典值统一为字符串） */
    modelValue?: string;
    /** 是否可清空 */
    clearable?: boolean;
    /** 是否禁用 */
    disabled?: boolean;
    /** 占位提示 */
    placeholder?: string;
  }>(),
  { modelValue: '', clearable: true, disabled: false, placeholder: '请选择' },
);

const emit = defineEmits<{
  /** 值变化，清空时回传空串 */
  (e: 'update:modelValue', value: string): void;
}>();

const dictStore = useDictStore();

/** 字典项转换为下拉选项 */
const options = computed(() =>
  (dictStore.items[props.dictCode] ?? []).map((item) => ({
    label: item.item_label,
    value: item.item_value,
  })),
);

/** 按需加载字典项，命中缓存时不会重复请求 */
async function loadDict(): Promise<void> {
  try {
    await dictStore.loadItems(props.dictCode);
  } catch {
    // 字典加载失败时下拉为空，由页面统一提示接口错误
  }
}

function onChange(value: unknown): void {
  emit('update:modelValue', value === null || value === undefined ? '' : String(value));
}

onMounted(() => {
  void loadDict();
});

watch(
  () => props.dictCode,
  () => {
    void loadDict();
  },
);
</script>