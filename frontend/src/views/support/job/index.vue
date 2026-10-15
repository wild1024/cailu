<template>
  <div class="job-page">
    <t-card :bordered="false" class="filter-card">
      <t-row :gutter="16">
        <t-col :span="2.4">
          <t-input v-model="query.job_name" placeholder="任务名称" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-input v-model="query.job_group" placeholder="任务分组" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-select v-model="query.status" placeholder="状态" clearable>
            <t-option :value="1" label="正常" />
            <t-option :value="0" label="停用" />
          </t-select>
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
          <t-button v-privilege="['support:job:create']" theme="primary" @click="openCreate">
            <template #icon><t-icon name="add" /></template>
            新增任务
          </t-button>
          <t-button theme="default" @click="loadList">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="job-page__count">共 {{ pagination.total }} 条</span>
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
        <template #status="{ row }">
          <t-switch
            :value="row.status === 1"
            :disabled="!hasPermission('support:job:update')"
            @change="() => toggleStatus(row)"
          />
        </template>
        <template #last_run_at="{ row }">{{ formatDateTime(row.last_run_at) }}</template>
        <template #next_run_at="{ row }">{{ formatDateTime(row.next_run_at) }}</template>
        <template #created_at="{ row }">{{ formatDateTime(row.created_at) }}</template>
        <template #op="{ row }">
          <t-space size="small">
            <t-link v-privilege="['support:job:update']" theme="primary" @click="openEdit(row)">
              编辑
            </t-link>
            <t-link v-privilege="['support:job:trigger']" theme="primary" @click="confirmTrigger(row)">
              执行一次
            </t-link>
            <t-link theme="primary" @click="openLogs(row)">执行日志</t-link>
            <t-link v-privilege="['support:job:delete']" theme="danger" @click="confirmRemove(row)">
              删除
            </t-link>
          </t-space>
        </template>
      </t-table>
    </t-card>

    <t-dialog
      v-model:visible="formVisible"
      :header="isEdit ? '编辑任务' : '新增任务'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: submitting }"
      width="580px"
      destroy-on-close
      @confirm="submitForm"
    >
      <t-form ref="formRef" :data="form" :rules="formRules" label-width="90px">
        <t-form-item label="任务名称" name="job_name">
          <t-input v-model="form.job_name" placeholder="请输入任务名称" />
        </t-form-item>
        <t-form-item label="任务分组" name="job_group">
          <t-input v-model="form.job_group" placeholder="请输入任务分组" />
        </t-form-item>
        <t-form-item label="cron 表达式" name="cron_expr">
          <t-input v-model="form.cron_expr" placeholder="如 0 0/5 * * * ?" />
        </t-form-item>
        <t-form-item label="调用目标" name="invoke_target">
          <t-input v-model="form.invoke_target" placeholder="请输入调用目标，如 jobs::clean_log" />
        </t-form-item>
        <t-form-item label="调用参数" name="params">
          <t-textarea v-model="form.params" placeholder="请输入调用参数" :autosize="{ minRows: 2 }" />
        </t-form-item>
        <t-form-item label="状态" name="status">
          <t-radio-group v-model="form.status">
            <t-radio :value="1">正常</t-radio>
            <t-radio :value="0">停用</t-radio>
          </t-radio-group>
        </t-form-item>
        <t-form-item label="备注" name="remark">
          <t-textarea v-model="form.remark" placeholder="请输入备注" :maxlength="200" />
        </t-form-item>
      </t-form>
    </t-dialog>

    <t-drawer v-model:visible="logsVisible" :header="`执行日志 - ${currentRow?.job_name ?? ''}`" size="900px" :footer="false">
      <div class="table-toolbar">
        <t-space>
          <t-button theme="default" @click="loadLogs">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="table-toolbar-count">共 {{ logPagination.total }} 条</span>
      </div>
      <t-table
        row-key="id"
        :data="logs"
        :columns="logColumns"
        :loading="logLoading"
        :pagination="logPagination"
        :hover="true"
        :stripe="true"
        @page-change="onLogPageChange"
      >
        <template #created_at="{ row }">{{ formatDateTime(row.created_at) }}</template>
        <template #status="{ row }">
          <t-tag :theme="row.status === 1 ? 'success' : 'danger'" variant="light">
            {{ row.status === 1 ? '成功' : '失败' }}
          </t-tag>
        </template>
        <template #op="{ row }">
          <t-link theme="primary" @click="openLogDetail(row)">详情</t-link>
        </template>
      </t-table>
    </t-drawer>

    <t-drawer v-model:visible="logDetailVisible" header="执行日志详情" size="560px" :footer="false">
      <t-descriptions v-if="currentLog" :column="1" bordered>
        <t-descriptions-item label="任务名称">{{ currentLog.job_name }}</t-descriptions-item>
        <t-descriptions-item label="调用目标">{{ currentLog.invoke_target }}</t-descriptions-item>
        <t-descriptions-item label="执行结果">
          <t-tag :theme="currentLog.status === 1 ? 'success' : 'danger'" variant="light">
            {{ currentLog.status === 1 ? '成功' : '失败' }}
          </t-tag>
        </t-descriptions-item>
        <t-descriptions-item label="开始时间">{{ formatDateTime(currentLog.started_at) }}</t-descriptions-item>
        <t-descriptions-item label="结束时间">{{ formatDateTime(currentLog.finished_at) }}</t-descriptions-item>
        <t-descriptions-item label="耗时">{{ currentLog.duration_ms }} ms</t-descriptions-item>
        <t-descriptions-item label="执行消息">{{ currentLog.message || '-' }}</t-descriptions-item>
        <t-descriptions-item label="异常信息">
          <pre class="job-page__detail">{{ currentLog.error || '-' }}</pre>
        </t-descriptions-item>
      </t-descriptions>
    </t-drawer>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import {
  DialogPlugin,
  MessagePlugin,
  type FormInstanceFunctions,
  type FormRules,
  type PrimaryTableCol,
} from 'tdesign-vue-next';
import {
  changeJobStatus,
  createJob,
  deleteJob,
  getJob,
  pageJobs,
  pageJobLogs,
  triggerJob,
  updateJob,
  type JobLogRes,
  type JobRes,
} from '@/api/support/job-api';
import type { StatusValue } from '@/api/system/user-api';
import { ApiError } from '@/api/request';
import { useUserStore } from '@/stores/user';
import { formatDateTime } from '@/utils/format';

/** 表格分页事件载荷 */
interface TablePageInfo {
  /** 当前页码 */
  current: number;
  /** 每页条数 */
  pageSize: number;
}

/** 查询条件 */
interface JobQuery {
  /** 任务名称 */
  job_name: string;
  /** 任务分组 */
  job_group: string;
  /** 状态 */
  status: StatusValue | undefined;
}

/** 新增/编辑表单 */
interface JobForm {
  /** 任务 id，新增时为空 */
  id: string;
  /** 任务名称 */
  job_name: string;
  /** 任务分组 */
  job_group: string;
  /** cron 表达式 */
  cron_expr: string;
  /** 调用目标 */
  invoke_target: string;
  /** 调用参数 */
  params: string;
  /** 状态 */
  status: StatusValue;
  /** 备注 */
  remark: string;
}

const userStore = useUserStore();
const { hasPermission } = userStore;

const query = reactive<JobQuery>({ job_name: '', job_group: '', status: undefined });
const rows = ref<JobRes[]>([]);
const loading = ref(false);
const pagination = reactive({ current: 1, pageSize: 10, total: 0 });

const columns: PrimaryTableCol[] = [
  { colKey: 'job_name', title: '任务名称', width: 180 },
  { colKey: 'job_group', title: '分组', width: 120 },
  { colKey: 'cron_expr', title: 'cron 表达式', width: 170 },
  { colKey: 'invoke_target', title: '调用目标', ellipsis: true },
  { colKey: 'status', title: '状态', width: 90, cell: 'status' },
  { colKey: 'last_run_at', title: '上次执行', width: 180, cell: 'last_run_at' },
  { colKey: 'next_run_at', title: '下次执行', width: 180, cell: 'next_run_at' },
  { colKey: 'op', title: '操作', width: 260, fixed: 'right' },
];

const formVisible = ref(false);
const isEdit = ref(false);
const submitting = ref(false);
const formRef = ref<FormInstanceFunctions | null>(null);
const form = reactive<JobForm>({
  id: '',
  job_name: '',
  job_group: '',
  cron_expr: '',
  invoke_target: '',
  params: '',
  status: 1,
  remark: '',
});
const formRules: FormRules<JobForm> = {
  job_name: [{ required: true, message: '请输入任务名称' }],
  cron_expr: [{ required: true, message: '请输入 cron 表达式' }],
  invoke_target: [{ required: true, message: '请输入调用目标' }],
};

const currentRow = ref<JobRes | null>(null);
const logsVisible = ref(false);
const logLoading = ref(false);
const logs = ref<JobLogRes[]>([]);
const logPagination = reactive({ current: 1, pageSize: 10, total: 0 });
const logColumns: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '记录时间', width: 180, cell: 'created_at' },
  { colKey: 'invoke_target', title: '调用目标', width: 200, ellipsis: true },
  { colKey: 'status', title: '结果', width: 90, cell: 'status' },
  { colKey: 'duration_ms', title: '耗时(ms)', width: 100 },
  { colKey: 'message', title: '消息', ellipsis: true },
  { colKey: 'op', title: '操作', width: 80, fixed: 'right' },
];

const logDetailVisible = ref(false);
const currentLog = ref<JobLogRes | null>(null);

/** 统一展示接口错误 */
function showError(message: string): void {
  void MessagePlugin.error(message);
}

async function loadList(): Promise<void> {
  loading.value = true;
  try {
    const data = await pageJobs({
      page: pagination.current,
      page_size: pagination.pageSize,
      job_name: query.job_name || undefined,
      job_group: query.job_group || undefined,
      status: query.status,
    });
    rows.value = data.list;
    pagination.total = data.total;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '任务列表加载失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch(): void {
  pagination.current = 1;
  void loadList();
}

function handleReset(): void {
  query.job_name = '';
  query.job_group = '';
  query.status = undefined;
  handleSearch();
}

function handlePageChange(pageInfo: TablePageInfo): void {
  pagination.current = pageInfo.current;
  pagination.pageSize = pageInfo.pageSize;
  void loadList();
}

function openCreate(): void {
  isEdit.value = false;
  Object.assign(form, {
    id: '',
    job_name: '',
    job_group: '',
    cron_expr: '',
    invoke_target: '',
    params: '',
    status: 1,
    remark: '',
  });
  formVisible.value = true;
}

async function openEdit(row: JobRes): Promise<void> {
  try {
    const detail = await getJob(row.id);
    isEdit.value = true;
    Object.assign(form, {
      id: detail.id,
      job_name: detail.job_name,
      job_group: detail.job_group,
      cron_expr: detail.cron_expr,
      invoke_target: detail.invoke_target,
      params: detail.params,
      status: detail.status,
      remark: detail.remark,
    });
    formVisible.value = true;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '任务详情加载失败');
  }
}

async function submitForm(): Promise<void> {
  const valid = await formRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload = {
    job_name: form.job_name,
    job_group: form.job_group,
    cron_expr: form.cron_expr,
    invoke_target: form.invoke_target,
    params: form.params,
    status: form.status,
    remark: form.remark,
  };
  submitting.value = true;
  try {
    if (isEdit.value) {
      await updateJob(form.id, payload);
      void MessagePlugin.success('任务已更新');
    } else {
      await createJob(payload);
      void MessagePlugin.success('任务已创建');
    }
    formVisible.value = false;
    await loadList();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    submitting.value = false;
  }
}

/** 启停任务 */
function toggleStatus(row: JobRes): void {
  const next: StatusValue = row.status === 1 ? 0 : 1;
  const action = next === 1 ? '启用' : '停用';
  const dialog = DialogPlugin.confirm({
    header: `${action}任务`,
    body: `确认${action}任务「${row.job_name}」？`,
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await changeJobStatus(row.id, next);
          void MessagePlugin.success(`已${action}`);
          await loadList();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : `${action}失败`);
        }
      })();
    },
  });
}

/** 手动执行一次任务 */
function confirmTrigger(row: JobRes): void {
  const dialog = DialogPlugin.confirm({
    header: '手动执行',
    body: `确认立即执行任务「${row.job_name}」一次？`,
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await triggerJob(row.id);
          void MessagePlugin.success('已触发执行');
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '触发执行失败');
        }
      })();
    },
  });
}

function confirmRemove(row: JobRes): void {
  const dialog = DialogPlugin.confirm({
    header: '删除任务',
    body: `确认删除任务「${row.job_name}」？该操作不可恢复。`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteJob(row.id);
          void MessagePlugin.success('任务已删除');
          await loadList();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

async function loadLogs(): Promise<void> {
  if (!currentRow.value) {
    return;
  }
  logLoading.value = true;
  try {
    const data = await pageJobLogs({
      page: logPagination.current,
      page_size: logPagination.pageSize,
      job_id: currentRow.value.id,
    });
    logs.value = data.list;
    logPagination.total = data.total;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '执行日志加载失败');
  } finally {
    logLoading.value = false;
  }
}

function onLogPageChange(pageInfo: TablePageInfo): void {
  logPagination.current = pageInfo.current;
  logPagination.pageSize = pageInfo.pageSize;
  void loadLogs();
}

function openLogs(row: JobRes): void {
  currentRow.value = row;
  logs.value = [];
  logPagination.current = 1;
  logPagination.total = 0;
  logsVisible.value = true;
  void loadLogs();
}

function openLogDetail(row: JobLogRes): void {
  currentLog.value = row;
  logDetailVisible.value = true;
}

onMounted(() => {
  void loadList();
});
</script>

<style scoped lang="less">
.job-page {
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

  &__detail {
    margin: 0;
    word-break: break-all;
    white-space: pre-wrap;
  }
}
</style>
