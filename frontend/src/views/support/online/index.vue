<template>
  <div class="online-page">
    <t-card :bordered="false" class="filter-card">
      <t-row :gutter="16">
        <t-col :span="2.4">
          <t-input v-model="query.username" placeholder="用户名" clearable @enter="handleSearch" />
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
          <t-button
            v-privilege="['support:online:force']"
            theme="danger"
            variant="outline"
            :disabled="selectedIds.length === 0"
            :loading="isBatchLoading"
            @click="handleBatchForceLogout"
          >
            <template #icon><t-icon name="poweroff" /></template>
            批量下线
          </t-button>
          <t-button theme="default" @click="loadList">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="online-page__count">当前在线 {{ pagination.total }} 人</span>
      </div>

      <t-table
        row-key="token_id"
        :data="rows"
        :columns="columns"
        :loading="loading"
        :pagination="pagination"
        :hover="true"
        :stripe="true"
        :selected-row-keys="selectedIds"
        @select-change="handleSelectChange"
        @page-change="handlePageChange"
      >
        <template #ip="{ row }">{{ orDash(row.ip) }}</template>
        <template #location="{ row }">{{ orDash(row.location) }}</template>
        <template #browser="{ row }">{{ orDash(row.browser) }}</template>
        <template #os_type="{ row }">{{ orDash(row.os_type) }}</template>
        <template #login_time="{ row }">{{ formatDateTime(row.login_time) }}</template>
        <template #expire_at="{ row }">{{ formatDateTime(row.expire_at) }}</template>
        <template #op="{ row }">
          <t-link
            v-privilege="['support:online:force']"
            theme="danger"
            @click="confirmForceLogout(row)"
          >
            强制下线
          </t-link>
        </template>
      </t-table>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { DialogPlugin, MessagePlugin, type PrimaryTableCol } from 'tdesign-vue-next';
import { forceLogout, pageOnline, type OnlineRes } from '@/api/support/online-api';
import { ApiError } from '@/api/request';
import { formatDateTime } from '@/utils/format';

/** 表格分页事件载荷 */
interface TablePageInfo {
  /** 当前页码 */
  current: number;
  /** 每页条数 */
  pageSize: number;
}

/** 查询条件 */
interface OnlineQuery {
  /** 用户名（模糊） */
  username: string;
}

const query = reactive<OnlineQuery>({ username: '' });
const rows = ref<OnlineRes[]>([]);
const loading = ref(false);
const pagination = reactive({ current: 1, pageSize: 10, total: 0 });
/** 已勾选的会话标识（token_id） */
const selectedIds = ref<string[]>([]);
/** 批量下线请求进行中 */
const isBatchLoading = ref(false);

const columns: PrimaryTableCol[] = [
  { colKey: 'row-select', type: 'multiple', width: 50 },
  { colKey: 'username', title: '用户名', width: 140 },
  { colKey: 'ip', title: 'IP 地址', width: 150, cell: 'ip' },
  { colKey: 'location', title: '登录地点', width: 150, cell: 'location' },
  { colKey: 'browser', title: '浏览器', width: 140, ellipsis: true, cell: 'browser' },
  { colKey: 'os_type', title: '操作系统', width: 130, cell: 'os_type' },
  { colKey: 'login_time', title: '登录时间', width: 180, cell: 'login_time' },
  { colKey: 'expire_at', title: '到期时间', width: 180, cell: 'expire_at' },
  { colKey: 'op', title: '操作', width: 110, fixed: 'right', cell: 'op' },
];

/** 空字符串统一展示为 `-` */
function orDash(value: string): string {
  return value && value.trim() ? value : '-';
}

/** 统一展示接口错误 */
function showError(message: string): void {
  void MessagePlugin.error(message);
}

async function loadList(): Promise<void> {
  loading.value = true;
  try {
    const data = await pageOnline({
      page: pagination.current,
      page_size: pagination.pageSize,
      username: query.username || undefined,
    });
    rows.value = data.list;
    pagination.total = data.total;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '在线用户加载失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch(): void {
  pagination.current = 1;
  void loadList();
}

function handleReset(): void {
  query.username = '';
  handleSearch();
}

function handlePageChange(pageInfo: TablePageInfo): void {
  pagination.current = pageInfo.current;
  pagination.pageSize = pageInfo.pageSize;
  void loadList();
}

/** 同步多选结果 */
function handleSelectChange(selectedRowKeys: Array<string | number>): void {
  selectedIds.value = selectedRowKeys.map(String);
}

/** 强制下线二次确认 */
function confirmForceLogout(row: OnlineRes): void {
  const dialog = DialogPlugin.confirm({
    header: '强制下线',
    body: `确认将用户「${row.username}」强制下线？该操作会立即使其令牌失效。`,
    theme: 'danger',
    confirmBtn: { content: '强制下线', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await forceLogout(row.token_id);
          void MessagePlugin.success(`已将「${row.username}」强制下线`);
          await loadList();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '强制下线失败');
        }
      })();
    },
  });
}

/**
 * 批量强制下线。
 *
 * 后端仅提供单个会话下线接口，故在前端对已勾选的会话逐个调用 `forceLogout`，
 * 并统计成功/失败数量；失败的会话保留在提示中，便于用户重试。
 */
async function handleBatchForceLogout(): Promise<void> {
  if (selectedIds.value.length === 0) {
    return;
  }
  const targetIds = [...selectedIds.value];
  isBatchLoading.value = true;
  let successCount = 0;
  const failedUsers: string[] = [];
  for (const tokenId of targetIds) {
    const target = rows.value.find((item) => item.token_id === tokenId);
    try {
      await forceLogout(tokenId);
      successCount += 1;
    } catch {
      failedUsers.push(target?.username ?? tokenId);
    }
  }
  isBatchLoading.value = false;
  if (failedUsers.length > 0) {
    showError(`以下用户强制下线失败：${failedUsers.join('、')}`);
  }
  if (successCount > 0) {
    void MessagePlugin.success(`已强制下线 ${successCount} 个会话`);
  }
  selectedIds.value = [];
  await loadList();
}

onMounted(() => {
  void loadList();
});
</script>

<style scoped lang="less">
.online-page {
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

  &__count {
    font-size: 13px;
    color: var(--td-text-color-secondary);
  }
}
</style>
