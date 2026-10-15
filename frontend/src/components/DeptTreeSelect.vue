<template>
  <t-tree-select
    :value="modelValue || undefined"
    :data="treeData"
    :tree-props="treeProps"
    :loading="loading"
    :disabled="disabled"
    :clearable="clearable"
    :placeholder="placeholder"
    filterable
    @change="onChange"
  />
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { getDeptTree, type DeptNode } from '@/api/system/dept-api';
import { ApiError } from '@/api/request';

const props = withDefaults(
  defineProps<{
    /** 当前部门 id */
    modelValue?: string | null;
    /** 是否可清空 */
    clearable?: boolean;
    /** 是否禁用 */
    disabled?: boolean;
    /** 占位提示 */
    placeholder?: string;
  }>(),
  { modelValue: null, clearable: true, disabled: false, placeholder: '请选择部门' },
);

const emit = defineEmits<{
  /** 值变化 */
  (e: 'update:modelValue', value: string | undefined): void;
  /** 部门树加载失败 */
  (e: 'error', message: string): void;
}>();

const treeData = ref<DeptNode[]>([]);
const loading = ref(false);

/** 部门树的字段映射：id 为值、dept_name 为标签、children 为子节点 */
const treeProps = { keys: { value: 'id', label: 'dept_name', children: 'children' } };

/** 加载部门树，供下拉选择使用 */
async function loadTree(): Promise<void> {
  loading.value = true;
  try {
    treeData.value = await getDeptTree();
  } catch (error) {
    treeData.value = [];
    emit('error', error instanceof ApiError ? error.message : '部门树加载失败');
  } finally {
    loading.value = false;
  }
}

function onChange(value: unknown): void {
  emit('update:modelValue', typeof value === 'string' && value ? value : undefined);
}

onMounted(() => {
  void loadTree();
});

watch(
  () => props.disabled,
  (disabled) => {
    // 首次展开（从禁用变为可用）时若无数据则补拉，避免无谓请求
    if (!disabled && treeData.value.length === 0) {
      void loadTree();
    }
  },
);
</script>