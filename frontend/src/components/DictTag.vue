<template>
  <t-tag v-if="label" :theme="theme" variant="light">{{ label }}</t-tag>
  <span v-else>-</span>
</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue';
import { useDictStore } from '@/stores/dict';

/** 标签主题色 */
type TagTheme = 'default' | 'primary' | 'success' | 'warning' | 'danger';

const props = withDefaults(
  defineProps<{
    /** 字典编码 */
    dictCode: string;
    /** 字典值 */
    value?: string | number | null;
    /** 标签主题色 */
    theme?: TagTheme;
  }>(),
  { value: null, theme: 'primary' },
);

const dictStore = useDictStore();

/** 字典标签，字典项未加载时回退为原始值 */
const label = computed(() => dictStore.getLabel(props.dictCode, props.value ?? undefined));

/** 按需加载字典项，命中缓存时不会重复请求 */
async function loadDict(): Promise<void> {
  try {
    await dictStore.loadItems(props.dictCode);
  } catch {
    // 字典加载失败时仅展示原始值，避免影响列表渲染
  }
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