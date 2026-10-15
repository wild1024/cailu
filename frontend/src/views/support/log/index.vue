<template>
  <div class="log-page">
    <!-- 筛选区：tabs 切换日志类型 + 通用查询条件 -->
    <t-card :bordered="false" class="filter-card">
      <t-tabs v-model="activeTab" @change="handleTabChange">
        <t-tab-panel v-for="tab in logTabs" :key="tab.value" :value="tab.value" :label="tab.label" />
      </t-tabs>

      <t-row :gutter="16" class="filter-row">
        <t-col :span="2.4">
          <t-input v-model="query.username" placeholder="用户名" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-date-range-picker
            :value="timeRange"
            allow-input
            clearable
            :placeholder="['开始时间', '结束时间']"
            @change="handleTimeChange"
          />
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

    <!-- 表格区：toolbar + 分页表格 -->
    <t-card :bordered="false" class="table-card">
      <div class="toolbar">
        <t-space>
          <!-- 清理仅操作日志 tab 可用 -->
          <t-button
            v-if="activeTab === 'operation'"
            v-privilege="['support:log:clean']"
            theme="danger"
            variant="outline"
            @click="openClean"
          >
            <template #icon><t-icon name="clear" /></template>
            清理日志
          </t-button>
          <t-button theme="default" @click="loadList">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="log-page__count">共 {{ pagination.total }} 条</span>
      </div>

      <t-table
        row-key="id"
        :data="rows"
        :columns="columns"
        :loading="loading"
        :pagination="pagination"
        :hover="true"
        :stripe="true"
        cell-empty-content="-"
        @page-change="handlePageChange"
      >
        <!-- 操作日志专用单元格 -->
        <template #request_method="{ row }">
          <t-tag :theme="methodTheme(toText(row.request_method))" variant="light">
            {{ toText(row.request_method) }}
          </t-tag>
        </template>
        <template #status_code="{ row }">
          <t-tag :theme="statusCodeTheme(Number(row.status_code))" variant="light">
            {{ toText(row.status_code) }}
          </t-tag>
        </template>
        <template #op="{ row }">
          <t-link theme="primary" @click="openDetail(row)">详情</t-link>
        </template>

        <!-- 通用单元格：时间 / IP / 状态 / 文本 -->
        <template #created_at="{ row }">{{ formatDateTime(toText(row.created_at)) }}</template>
        <template #ip="{ row }">{{ orDash(toText(row.ip)) }}</template>
        <template #text="{ row, col }">{{ cellText(row, col) }}</template>

        <!-- 字典映射单元格：编码 -> 中文 -->
        <template #dict_auth_type="{ row }">
          {{ dictLabel(DICT_LOG_AUTH_TYPE, toText(row.auth_type)) }}
        </template>
        <template #dict_operation="{ row }">
          {{ dictLabel(DICT_LOG_OPERATION, toText(row.operation)) }}
        </template>
        <template #dict_target_type="{ row }">
          {{ dictLabel(DICT_LOG_TARGET_TYPE, toText(row.target_type)) }}
        </template>
        <template #dict_event_type="{ row }">
          {{ dictLabel(DICT_LOG_SECURITY_EVENT, toText(row.event_type)) }}
        </template>
        <template #dict_change_type="{ row }">
          {{ dictLabel(DICT_LOG_PASSWORD_CHANGE_TYPE, toText(row.change_type)) }}
        </template>
        <template #auth_status="{ row }">
          <t-tag v-if="isBlank(row.auth_status)" variant="light">-</t-tag>
          <t-tag v-else :theme="isSuccessStatus(row.auth_status) ? 'success' : 'danger'" variant="light">
            {{ isSuccessStatus(row.auth_status) ? '成功' : '失败' }}
          </t-tag>
        </template>
        <template #change_status="{ row }">
          <t-tag v-if="isBlank(row.change_status)" variant="light">-</t-tag>
          <t-tag v-else :theme="isSuccessStatus(row.change_status) ? 'success' : 'danger'" variant="light">
            {{ isSuccessStatus(row.change_status) ? '成功' : '失败' }}
          </t-tag>
        </template>
      </t-table>
    </t-card>

    <!-- 详情抽屉：仅操作日志存在详情接口 -->
    <t-drawer
      v-model:visible="detailVisible"
      header="日志详情"
      size="640px"
      :footer="false"
      destroy-on-close
    >
      <t-loading :loading="detailLoading">
        <t-descriptions v-if="detail" :column="1" bordered>
          <t-descriptions-item label="操作">{{ orDash(detail.operation) }}</t-descriptions-item>
          <t-descriptions-item label="操作人">{{ orDash(detail.username) }}</t-descriptions-item>
          <t-descriptions-item label="请求">
            {{ detail.request_method }} {{ detail.request_path }}
          </t-descriptions-item>
          <t-descriptions-item label="状态码">
            <t-tag :theme="statusCodeTheme(detail.status_code)" variant="light">
              {{ detail.status_code }}
            </t-tag>
          </t-descriptions-item>
          <t-descriptions-item label="耗时">{{ detail.duration_ms }} ms</t-descriptions-item>
          <t-descriptions-item label="客户端 IP">{{ orDash(detail.ip) }}</t-descriptions-item>
          <t-descriptions-item label="目标资源">
            {{ orDash(detail.target_type) }} / {{ orDash(detail.target_id) }}
          </t-descriptions-item>
          <t-descriptions-item label="请求链路">{{ orDash(detail.request_id) }}</t-descriptions-item>
          <t-descriptions-item label="发生时间">{{ formatDateTime(detail.created_at) }}</t-descriptions-item>
          <t-descriptions-item label="详情">{{ orDash(detail.detail) }}</t-descriptions-item>
          <t-descriptions-item label="User-Agent">{{ orDash(detail.user_agent) }}</t-descriptions-item>
          <t-descriptions-item label="请求参数">
            <pre class="log-page__params">{{ requestParamsText }}</pre>
          </t-descriptions-item>
          <t-descriptions-item label="响应数据">
            <pre class="log-page__params">{{ responseDataText }}</pre>
          </t-descriptions-item>
        </t-descriptions>
      </t-loading>
    </t-drawer>

    <t-dialog
      v-model:visible="cleanVisible"
      header="清理历史日志"
      :confirm-btn="{ content: '确定清理', theme: 'danger', loading: cleaning }"
      width="440px"
      destroy-on-close
      @confirm="submitClean"
    >
      <t-form :data="cleanForm" label-width="110px">
        <t-form-item label="保留天数">
          <t-input-number v-model="cleanForm.days" :min="1" :max="3650" theme="normal" />
        </t-form-item>
        <t-form-item label-width="0">
          <span class="log-page__count">将删除 {{ cleanForm.days }} 天前的全部操作日志</span>
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { MessagePlugin, type PrimaryTableCol } from 'tdesign-vue-next';
import {
  cleanLogs,
  getLog,
  pageAuditLogs,
  pageAuthLogs,
  pageLogs,
  pagePasswordLogs,
  pageSecurityLogs,
  type LogDetailRes,
  type LogPageQuery,
  type RawLogPageQuery,
  type RawLogRow,
} from '@/api/support/log-api';
import { ApiError, type PageData } from '@/api/request';
import { formatDateTime } from '@/utils/format';
import { useDictStore } from '@/stores/dict';

/** 日志字段对应的字典编码 */
const DICT_LOG_AUTH_TYPE = 'log_auth_type';
const DICT_LOG_OPERATION = 'log_operation';
const DICT_LOG_TARGET_TYPE = 'log_target_type';
const DICT_LOG_SECURITY_EVENT = 'log_security_event';
const DICT_LOG_PASSWORD_CHANGE_TYPE = 'log_password_change_type';

/** 日志页面需预加载的字典编码集合 */
const LOG_DICT_CODES = [
  DICT_LOG_AUTH_TYPE,
  DICT_LOG_OPERATION,
  DICT_LOG_TARGET_TYPE,
  DICT_LOG_SECURITY_EVENT,
  DICT_LOG_PASSWORD_CHANGE_TYPE,
];

/** 日志类型 tab 枚举值 */
type LogTab = 'operation' | 'login' | 'audit' | 'security' | 'password';

/** tab 配置 */
const logTabs: { value: LogTab; label: string }[] = [
  { value: 'operation', label: '操作日志' },
  { value: 'login', label: '登录日志' },
  { value: 'audit', label: '审计日志' },
  { value: 'security', label: '安全日志' },
  { value: 'password', label: '改密日志' },
];

/** 非操作日志的分页查询器：按 tab 路由到对应后端接口 */
const rawFetchers: Record<
  Exclude<LogTab, 'operation'>,
  (query: RawLogPageQuery) => Promise<PageData<RawLogRow>>
> = {
  login: pageAuthLogs,
  audit: pageAuditLogs,
  security: pageSecurityLogs,
  password: pagePasswordLogs,
};

/** 表格分页事件载荷 */
interface TablePageInfo {
  /** 当前页码 */
  current: number;
  /** 每页条数 */
  pageSize: number;
}

/** 操作日志列 */
const OPERATION_COLUMNS: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '发生时间', width: 180, cell: 'created_at' },
  { colKey: 'username', title: '操作人', width: 130, cell: 'text' },
  { colKey: 'operation', title: '操作', ellipsis: true, cell: 'text' },
  { colKey: 'request_method', title: '方法', width: 90, cell: 'request_method' },
  { colKey: 'request_path', title: '请求路径', width: 220, ellipsis: true, cell: 'text' },
  { colKey: 'status_code', title: '状态码', width: 90, cell: 'status_code' },
  { colKey: 'ip', title: '客户端 IP', width: 140, cell: 'ip' },
  { colKey: 'duration_ms', title: '耗时(ms)', width: 100, cell: 'text' },
  { colKey: 'op', title: '操作', width: 80, fixed: 'right', cell: 'op' },
];

/** 登录（认证）日志列 */
const LOGIN_COLUMNS: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '时间', width: 180, cell: 'created_at' },
  { colKey: 'username', title: '用户名', width: 130, cell: 'text' },
  { colKey: 'auth_type', title: '认证类型', width: 110, cell: 'dict_auth_type' },
  { colKey: 'auth_status', title: '状态', width: 100, cell: 'auth_status' },
  { colKey: 'ip', title: 'IP 地址', width: 140, cell: 'ip' },
  { colKey: 'user_agent', title: 'User-Agent', width: 260, ellipsis: true, cell: 'text' },
  { colKey: 'error_message', title: '失败原因', width: 180, ellipsis: true, cell: 'text' },
];

/** 审计日志列 */
const AUDIT_COLUMNS: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '时间', width: 180, cell: 'created_at' },
  { colKey: 'username', title: '用户名', width: 130, cell: 'text' },
  { colKey: 'operation', title: '操作', width: 180, ellipsis: true, cell: 'dict_operation' },
  { colKey: 'target_type', title: '目标类型', width: 140, cell: 'dict_target_type' },
  { colKey: 'target_id', title: '目标 ID', width: 180, ellipsis: true, cell: 'text' },
  { colKey: 'ip', title: 'IP 地址', width: 140, cell: 'ip' },
  { colKey: 'request_id', title: '请求链路', width: 200, ellipsis: true, cell: 'text' },
];

/** 安全日志列 */
const SECURITY_COLUMNS: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '时间', width: 180, cell: 'created_at' },
  { colKey: 'username', title: '用户名', width: 130, cell: 'text' },
  { colKey: 'event_type', title: '事件类型', width: 160, cell: 'dict_event_type' },
  { colKey: 'ip', title: 'IP 地址', width: 140, cell: 'ip' },
  { colKey: 'detail', title: '详情', ellipsis: true, cell: 'text' },
];

/** 改密日志列 */
const PASSWORD_COLUMNS: PrimaryTableCol[] = [
  { colKey: 'created_at', title: '时间', width: 180, cell: 'created_at' },
  { colKey: 'username', title: '用户名', width: 130, cell: 'text' },
  { colKey: 'change_type', title: '变更类型', width: 140, cell: 'dict_change_type' },
  { colKey: 'change_status', title: '状态', width: 100, cell: 'change_status' },
  { colKey: 'ip', title: 'IP 地址', width: 140, cell: 'ip' },
  { colKey: 'error_message', title: '说明', width: 180, ellipsis: true, cell: 'text' },
];

/** tab 与列配置映射 */
const tabColumns: Record<LogTab, PrimaryTableCol[]> = {
  operation: OPERATION_COLUMNS,
  login: LOGIN_COLUMNS,
  audit: AUDIT_COLUMNS,
  security: SECURITY_COLUMNS,
  password: PASSWORD_COLUMNS,
};

const activeTab = ref<LogTab>('operation');
const dictStore = useDictStore();
const query = reactive({ username: '' });
const timeRange = ref<string[]>([]);
const rows = ref<RawLogRow[]>([]);
const loading = ref(false);
const pagination = reactive({ current: 1, pageSize: 10, total: 0 });

/** 当前 tab 对应的表格列 */
const columns = computed<PrimaryTableCol[]>(() => tabColumns[activeTab.value]);

const detailVisible = ref(false);
const detailLoading = ref(false);
const detail = ref<LogDetailRes | null>(null);

const cleanVisible = ref(false);
const cleaning = ref(false);
const cleanForm = reactive({ days: 30 });

/** 请求参数文本，反序列化为格式化 JSON */
const requestParamsText = computed(() => formatJson(detail.value?.request_params));

/** 响应数据文本，反序列化为格式化 JSON */
const responseDataText = computed(() => formatJson(detail.value?.response_data));

/** 判定值是否为空（null / undefined / 空字符串） */
function isBlank(value: unknown): boolean {
  return value === undefined || value === null || value === '';
}

/** 空字符串统一展示为 `-` */
function orDash(value: string): string {
  return value && value.trim() ? value : '-';
}

/** 字典标签：命中返回中文，未命中回退原始编码，空值展示为 `-` */
function dictLabel(dictCode: string, value: string): string {
  if (!value) {
    return '-';
  }
  return dictStore.getLabel(dictCode, value);
}

/** 预加载日志字典；单个失败不影响其余字典与列表渲染 */
async function preloadLogDicts(): Promise<void> {
  await Promise.all(
    LOG_DICT_CODES.map((code) =>
      dictStore.loadItems(code).catch(() => []),
    ),
  );
}

/** 未知字段安全转字符串，空值返回空串（由表格占位符兜底） */
function toText(value: unknown): string {
  if (isBlank(value)) {
    return '';
  }
  return typeof value === 'string' ? value : String(value);
}

/** 通用单元格文本：支持任意 JSON 行字段 */
function cellText(row: RawLogRow, col: { colKey?: string }): string {
  const key = col?.colKey;
  if (!key) {
    return '';
  }
  const value = row[key];
  if (isBlank(value)) {
    return '';
  }
  return typeof value === 'object' ? JSON.stringify(value) : String(value);
}

/** 判定状态是否为成功（兼容数字 / 字符串 / 布尔） */
function isSuccessStatus(value: unknown): boolean {
  return value === 1 || value === '1' || value === true || value === 'success' || value === 'SUCCESS';
}

/** 请求方法标签主题色 */
function methodTheme(method: string): 'primary' | 'success' | 'warning' | 'danger' {
  if (method === 'POST') {
    return 'success';
  }
  if (method === 'PUT' || method === 'PATCH') {
    return 'warning';
  }
  return method === 'DELETE' ? 'danger' : 'primary';
}

/** 状态码标签主题色 */
function statusCodeTheme(statusCode: number): 'success' | 'danger' {
  return statusCode >= 200 && statusCode < 400 ? 'success' : 'danger';
}

/** 将任意 JSON 值格式化为可读文本，空值返回占位符 */
function formatJson(value: unknown): string {
  if (value === undefined || value === null) {
    return '-';
  }
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

/** 统一展示接口错误 */
function showError(message: string): void {
  void MessagePlugin.error(message);
}

function handleTimeChange(value: unknown): void {
  timeRange.value = Array.isArray(value) ? value.map((item) => String(item)) : [];
}

async function loadList(): Promise<void> {
  loading.value = true;
  const tab = activeTab.value;
  const params: RawLogPageQuery = {
    page: pagination.current,
    page_size: pagination.pageSize,
    username: query.username || undefined,
    start_time: timeRange.value[0] || undefined,
    end_time: timeRange.value[1] || undefined,
  };
  try {
    if (tab === 'operation') {
      const data = await pageLogs(params as LogPageQuery);
      rows.value = data.list as unknown as RawLogRow[];
      pagination.total = data.total;
    } else {
      const data = await rawFetchers[tab](params);
      rows.value = data.list ?? [];
      pagination.total = data.total ?? 0;
    }
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '日志列表加载失败');
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
  timeRange.value = [];
  pagination.current = 1;
  void loadList();
}

function handleTabChange(): void {
  pagination.current = 1;
  rows.value = [];
  void loadList();
}

function handlePageChange(pageInfo: TablePageInfo): void {
  pagination.current = pageInfo.current;
  pagination.pageSize = pageInfo.pageSize;
  void loadList();
}

async function openDetail(row: RawLogRow): Promise<void> {
  detail.value = null;
  detailVisible.value = true;
  detailLoading.value = true;
  try {
    detail.value = await getLog(row.id);
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '日志详情加载失败');
  } finally {
    detailLoading.value = false;
  }
}

function openClean(): void {
  cleanForm.days = 30;
  cleanVisible.value = true;
}

async function submitClean(): Promise<void> {
  cleaning.value = true;
  try {
    const result = await cleanLogs(cleanForm.days);
    void MessagePlugin.success(`已清理 ${result.removed} 条日志`);
    cleanVisible.value = false;
    pagination.current = 1;
    await loadList();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '日志清理失败');
  } finally {
    cleaning.value = false;
  }
}

onMounted(() => {
  void preloadLogDicts();
  void loadList();
});
</script>

<style scoped lang="less">
.log-page {
  padding: 16px;

  .filter-card {
    margin-bottom: 12px;
  }

  .filter-row {
    margin-top: 12px;
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

  &__params {
    margin: 0;
    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
  }
}
</style>
