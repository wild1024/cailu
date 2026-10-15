<template>
  <span class="file-uploader">
    <t-button theme="primary" :loading="uploading" @click="openPicker">
      <template #icon><t-icon name="upload" /></template>
      上传文件
    </t-button>
    <input ref="inputRef" class="file-uploader__input" type="file" :accept="accept" @change="onFileChange" />
  </span>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { uploadFile, type FileRes } from '@/api/support/file-api';
import { ApiError } from '@/api/request';

const props = withDefaults(
  defineProps<{
    /** 文件分类，作为上传接口的 query 参数 */
    category?: string;
    /** 允许的文件类型（accept 属性） */
    accept?: string;
  }>(),
  { category: '', accept: '' },
);

const emit = defineEmits<{
  /** 上传成功 */
  (e: 'uploaded', file: FileRes): void;
  /** 上传失败 */
  (e: 'error', message: string): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const uploading = ref(false);

/** 触发隐藏的原生文件选择框 */
function openPicker(): void {
  inputRef.value?.click();
}

/** 选择文件后立即上传，并清空 input 以便重复选择同一文件 */
async function onFileChange(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) {
    return;
  }
  uploading.value = true;
  try {
    const result = await uploadFile(file, props.category || undefined);
    emit('uploaded', result);
  } catch (error) {
    emit('error', error instanceof ApiError ? error.message : '文件上传失败');
  } finally {
    uploading.value = false;
  }
}
</script>

<style scoped lang="less">
.file-uploader {
  display: inline-block;

  &__input {
    display: none;
  }
}
</style>