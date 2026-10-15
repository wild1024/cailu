<template>
  <div class="file-page">
    <t-card :bordered="false" class="filter-card">
      <t-row :gutter="16">
        <t-col :span="2.4">
          <t-input v-model="query.original_name" placeholder="文件名称" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-input v-model="query.category" placeholder="分类" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-input v-model="query.mime_type" placeholder="文件类型" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-space>
            <t-button theme="primary" @click="handleSearch">
              <template #icon><t-icon name="search" /></template>
              查询
            </t-button>
            <t-button theme="default" @click="handleReset">重置</t-button>
          </t-space>
        </t-col>
      </t-row>
    </t-card>

    <t-card :bordered="false" class="table-card">
      <div class="toolbar">
        <t-space>
          <t-input
            v-model="uploadCategory"
            v-privilege="['support:file:upload']"
            class="file-page__category"
            placeholder="上传分类（可选）"
            clearable
          />
          <file-uploader
            v-privilege="['support:file:upload']"
            :category="uploadCategory"
            @uploaded="onUploaded"
            @error="showError"
          />
          <t-button theme="default" @click="loadList">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="file-page__count">共 {{ pagination.total }} 条</span>
      </div>

      <t-table
        row-key="id"
        :data="rows"
        :columns="columns"
        :loading="loading"
        :pagination="pagination"
        :hover="true"
        :stripe="true"
        @page-change="handlePageChange"
      >
        <template #category="{ row }">{{ categoryLabel(row.category) }}</template>
        <template #file_size="{ row }">{{ formatFileSize(row.file_size) }}</template>
        <template #created_at="{ row }">{{ formatDateTime(row.created_at) }}</template>
        <template #op="{ row }">
          <t-space size="small">
            <t-link theme="primary" @click="handleDownload(row)">下载</t-link>
            <t-link v-privilege="['support:file:delete']" theme="danger" @click="confirmRemove(row)">
              删除
            </t-link>
          </t-space>
        </template>
      </t-table>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import {
  DialogPlugin,
  MessagePlugin,
  type PrimaryTableCol,
} from 'tdesign-vue-next';
import FileUploader from '@/components/FileUploader.vue';
import {
  deleteFile,
  downloadFileBlob,
  pageFiles,
  type FileRes,
} from '@/api/support/file-api';
import { ApiError } from '@/api/request';
import { formatDateTime, formatFileSize } from '@/utils/format';

/** 表格分页事件载荷 */
interface TablePageInfo {
  /** 当前页码 */
  current: number;
  /** 每页条数 */
  pageSize: number;
}

/** 查询条件 */
interface FileQuery {
  /** 原始文件名 */
  original_name: string;
  /** 文件分类 */
  category: string;
  /** MIME 类型 */
  mime_type: string;
}

const query = reactive<FileQuery>({ original_name: '', category: '', mime_type: '' });
const rows = ref<FileRes[]>([]);
const loading = ref(false);
const pagination = reactive({ current: 1, pageSize: 10, total: 0 });
/** 上传时携带的分类 */
const uploadCategory = ref('');

/** 分类中文名映射：库中分类存的是英文短码，展示统一转中文；未知分类回退原值 */
const CATEGORY_LABELS: Record<string, string> = {
  attachment: '附件',
  avatar: '头像',
  document: '文档',
};

/** 分类展示名：有中文映射显示中文，否则原样显示（用户自建分类） */
function categoryLabel(category: string | null | undefined): string {
  if (!category) {
    return '-';
  }
  return CATEGORY_LABELS[category] ?? category;
}

const columns: PrimaryTableCol[] = [
  { colKey: 'original_name', title: '文件名称', ellipsis: true },
  { colKey: 'category', title: '分类', width: 120, cell: 'category' },
  { colKey: 'mime_type', title: '文件类型', width: 180, ellipsis: true },
  { colKey: 'file_size', title: '大小', width: 110, cell: 'file_size' },
  { colKey: 'uploaded_by', title: '上传人', width: 140 },
  { colKey: 'created_at', title: '上传时间', width: 180, cell: 'created_at' },
  { colKey: 'op', title: '操作', width: 120, fixed: 'right' },
];

/** 统一展示接口错误 */
function showError(message: string): void {
  void MessagePlugin.error(message);
}

async function loadList(): Promise<void> {
  loading.value = true;
  try {
    const data = await pageFiles({
      page: pagination.current,
      page_size: pagination.pageSize,
      original_name: query.original_name || undefined,
      category: query.category || undefined,
      mime_type: query.mime_type || undefined,
    });
    rows.value = data.list;
    pagination.total = data.total;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '文件列表加载失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch(): void {
  pagination.current = 1;
  void loadList();
}

function handleReset(): void {
  query.original_name = '';
  query.category = '';
  query.mime_type = '';
  handleSearch();
}

function handlePageChange(pageInfo: TablePageInfo): void {
  pagination.current = pageInfo.current;
  pagination.pageSize = pageInfo.pageSize;
  void loadList();
}

function onUploaded(file: FileRes): void {
  void MessagePlugin.success(`「${file.original_name}」上传成功`);
  pagination.current = 1;
  void loadList();
}

/** 通过 blob 下载，保证请求携带鉴权头 */
async function handleDownload(row: FileRes): Promise<void> {
  try {
    const blob = await downloadFileBlob(row.id);
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = row.original_name || row.file_name || 'download';
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '文件下载失败');
  }
}

function confirmRemove(row: FileRes): void {
  const dialog = DialogPlugin.confirm({
    header: '删除文件',
    body: `确认删除文件「${row.original_name}」？该操作不可恢复。`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteFile(row.id);
          void MessagePlugin.success('文件已删除');
          await loadList();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

onMounted(() => {
  void loadList();
});
</script>

<style scoped lang="less">
.file-page {
  padding: 16px;

  .filter-card {
    margin-bottom: 12px;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  &__category {
    width: 180px;
  }

  &__count {
    font-size: 13px;
    color: var(--td-text-color-secondary);
  }
}
</style>
