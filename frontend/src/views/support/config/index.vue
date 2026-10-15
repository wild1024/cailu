<template>
  <div class="config-page">
    <!-- 分类 tab + 刷新缓存 -->
    <t-card :bordered="false" class="tab-card">
      <t-row align="middle" justify="space-between">
        <t-tabs v-model="activeCategory">
          <t-tab-panel value="" label="全部" />
          <t-tab-panel
            v-for="category in categoryList"
            :key="category"
            :value="category"
            :label="categoryLabel(category)"
          />
        </t-tabs>
        <t-space>
          <t-button
            theme="primary"
            variant="outline"
            :loading="reloadLoading"
            v-privilege="['support:config:update']"
            @click="handleRefreshCache"
          >
            <template #icon><t-icon name="refresh" /></template>
            刷新运行时配置
          </t-button>
        </t-space>
      </t-row>
    </t-card>

    <t-card :bordered="false" class="list-card">
      <div class="list-toolbar">
        <t-space>
          <t-input
            v-model="searchKeyword"
            placeholder="搜索配置Key或名称"
            clearable
            style="width: 280px"
          >
            <template #prefix-icon><t-icon name="search" /></template>
          </t-input>
          <t-button
            theme="primary"
            v-privilege="['support:config:create']"
            @click="openCreate"
          >
            <template #icon><t-icon name="add" /></template>
            新增
          </t-button>
        </t-space>
        <span class="list-toolbar-count">共 {{ filteredData.length }} 项</span>
      </div>

      <div v-for="section in sections" :key="section.category" class="config-group">
        <div v-if="section.showHeader" class="config-group-header">
          <t-icon name="folder" />
          <span class="config-group-title">{{ section.label }}</span>
          <span class="config-group-count">{{ section.items.length }} 项</span>
        </div>
        <div class="config-group-body">
          <div v-for="item in section.items" :key="item.id" class="config-item">
            <div class="config-header">
              <div class="config-title-area">
                <span class="config-title">{{ item.description || item.config_key }}</span>
                <span v-if="item.description" class="config-key-sub">{{ item.config_key }}</span>
                <t-tag v-if="item.ref_value" size="small" theme="warning" variant="light">
                  参考: {{ item.ref_value }}
                </t-tag>
              </div>
            </div>
            <div class="config-body">
              <div class="config-value-row">
                <div class="config-value-area">
                  <!-- 按值类型渲染对应的内联编辑器：类型口径见 editorKind -->
                  <t-input
                    v-if="editorKind(item.config_type) === 'text'"
                    v-model="textValueMap[item.id]"
                    :disabled="!item.is_modifiable"
                  />
                  <t-input-number
                    v-else-if="editorKind(item.config_type) === 'number'"
                    v-model="numberValueMap[item.id]"
                    :disabled="!item.is_modifiable"
                    theme="normal"
                  />
                  <t-switch
                    v-else-if="editorKind(item.config_type) === 'boolean'"
                    v-model="booleanValueMap[item.id]"
                    :disabled="!item.is_modifiable"
                    :label="['关闭', '开启']"
                  />
                  <t-textarea
                    v-else-if="editorKind(item.config_type) === 'json'"
                    v-model="textValueMap[item.id]"
                    :disabled="!item.is_modifiable"
                    :autosize="{ minRows: 2, maxRows: 6 }"
                  />
                  <t-input v-else v-model="textValueMap[item.id]" :disabled="!item.is_modifiable" />
                </div>
                <div class="config-actions">
                  <template v-if="item.is_modifiable">
                    <t-button
                      v-privilege="['support:config:update']"
                      theme="primary"
                      size="small"
                      :loading="savingMap[item.id]"
                      @click="handleSave(item)"
                    >
                      保存
                    </t-button>
                    <t-link
                      v-privilege="['support:config:update']"
                      theme="primary"
                      @click="openEdit(item)"
                    >
                      编辑
                    </t-link>
                    <t-link
                      v-privilege="['support:config:delete']"
                      theme="danger"
                      @click="confirmRemove(item)"
                    >
                      <t-icon name="delete" />
                    </t-link>
                  </template>
                  <t-tag v-else size="small" theme="default" variant="light">系统配置</t-tag>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <t-empty v-if="filteredData.length === 0" description="暂无配置数据" />
    </t-card>

    <!-- 新增/编辑弹窗 -->
    <t-dialog
      v-model:visible="formVisible"
      :header="isEdit ? '编辑参数' : '新增参数'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: submitting }"
      width="620px"
      destroy-on-close
      @confirm="submitForm"
    >
      <t-form ref="formRef" :data="form" :rules="formRules" label-width="100px">
        <t-form-item label="配置键" name="config_key">
          <t-input v-model="form.config_key" placeholder="请输入配置键" :disabled="isEdit" />
        </t-form-item>
        <t-form-item label="配置值" name="config_value">
          <t-textarea v-model="form.config_value" placeholder="请输入配置值" :autosize="{ minRows: 2 }" />
        </t-form-item>
        <t-form-item label="值类型" name="config_type">
          <t-select
            v-model="form.config_type"
            :options="enums.options('config_type')"
            placeholder="请选择值类型"
          />
        </t-form-item>
        <t-form-item label="配置分类" name="category">
          <t-input v-model="form.category" placeholder="请输入配置分类" />
        </t-form-item>
        <t-form-item label="配置级别" name="config_level">
          <t-input-number v-model="form.config_level" :min="0" :max="9999" theme="normal" />
        </t-form-item>
        <t-form-item label="说明" name="description">
          <t-input v-model="form.description" placeholder="请输入配置说明" />
        </t-form-item>
        <t-form-item label="引用配置键" name="ref_value">
          <t-input v-model="form.ref_value" placeholder="请输入引用的其他配置键" />
        </t-form-item>
        <t-form-item label="可选值" name="select_value">
          <t-input v-model="form.select_value" placeholder="多项以逗号分隔" />
        </t-form-item>
        <t-form-item label="排序" name="sort_order">
          <t-input-number v-model="form.sort_order" :min="0" :max="9999" theme="normal" />
        </t-form-item>
        <t-form-item label="允许修改" name="is_modifiable">
          <t-switch v-model="form.is_modifiable" />
        </t-form-item>
        <t-form-item label="加密存储" name="is_encrypted">
          <t-switch v-model="form.is_encrypted" />
        </t-form-item>
        <t-form-item label="重启生效" name="is_restart">
          <t-switch v-model="form.is_restart" />
        </t-form-item>
        <t-form-item label="状态" name="status">
          <t-radio-group v-model="form.status">
            <t-radio :value="1">正常</t-radio>
            <t-radio :value="0">停用</t-radio>
          </t-radio-group>
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import {
  DialogPlugin,
  MessagePlugin,
  type FormInstanceFunctions,
  type FormRules,
} from 'tdesign-vue-next';
import {
  createConfig,
  deleteConfig,
  getConfig,
  pageConfigs,
  reloadRuntimeConfig,
  updateConfig,
  type ConfigRes,
} from '@/api/support/config-api';
import type { StatusValue } from '@/api/system/user-api';
import { ApiError } from '@/api/request';
import { useDictStore } from '@/stores/dict';
import { useModuleEnums } from '@/composables/useModuleEnums';

/** 分类视图按前端分组过滤需全量数据，这里一次性拉取较大的分页 */
const FULL_PAGE_SIZE = 1000;

/** 系统参数分类字典编码：用于把 category 编码渲染为中文页签 */
const DICT_CONFIG_CATEGORY = 'config_category';

/** 新增/编辑表单 */
interface ConfigForm {
  /** 配置 id，新增时为空 */
  id: string;
  /** 配置键 */
  config_key: string;
  /** 配置值 */
  config_value: string;
  /** 值类型 */
  config_type: string;
  /** 说明 */
  description: string;
  /** 分类 */
  category: string;
  /** 排序值 */
  sort_order: number;
  /** 配置级别 */
  config_level: number;
  /** 允许修改 */
  is_modifiable: boolean;
  /** 加密存储 */
  is_encrypted: boolean;
  /** 重启生效 */
  is_restart: boolean;
  /** 状态 */
  status: StatusValue;
  /** 引用配置键 */
  ref_value: string;
  /** 可选值 */
  select_value: string;
}

/** 配置分组（“全部”视图按分类分组，分类视图仅一组且不显示表头） */
interface ConfigSection {
  /** 分类标识 */
  category: string;
  /** 分类展示名 */
  label: string;
  /** 分组内的配置项 */
  items: ConfigRes[];
  /** 是否显示分组表头 */
  showHeader: boolean;
}

/** 值类型：单一事实源在后端 `GET /support/config/enums`，此处只做展示映射 */
const enums = useModuleEnums('support/config');

/** 值类型 → 编辑控件类型（STRING 文本 / NUMBER 数字 / BOOLEAN 开关 / JSON 多行） */
function editorKind(configType: string): 'text' | 'number' | 'boolean' | 'json' | 'unknown' {
  switch (configType) {
    case 'STRING':
      return 'text';
    case 'NUMBER':
      return 'number';
    case 'BOOLEAN':
      return 'boolean';
    case 'JSON':
      return 'json';
    default:
      // 未知类型（后端白名单外、历史数据）降级为文本编辑，保证参数仍可维护
      return 'unknown';
  }
}

const allData = ref<ConfigRes[]>([]);
const activeCategory = ref('');
const searchKeyword = ref('');
const reloadLoading = ref(false);
const dictStore = useDictStore();

/** 内联编辑值：按 config_type 分别存储，避免跨类型混用 */
const textValueMap = reactive<Record<string, string>>({});
const numberValueMap = reactive<Record<string, number>>({});
const booleanValueMap = reactive<Record<string, boolean>>({});
const savingMap = reactive<Record<string, boolean>>({});

/** 分类展示名：优先取字典中文，未命中回退原始编码；空分类归为“未分类” */
function categoryLabel(category: string): string {
  if (!category) {
    return '未分类';
  }
  return dictStore.getLabel(DICT_CONFIG_CATEGORY, category);
}

/** 从配置列表中去重得到分类集合 */
const categoryList = computed<string[]>(() => {
  const seen = new Set<string>();
  for (const item of allData.value) {
    if (item.category) {
      seen.add(item.category);
    }
  }
  return Array.from(seen);
});

/** 按当前分类 Tab 与搜索关键字过滤后的数据 */
const filteredData = computed<ConfigRes[]>(() => {
  let list = allData.value;
  if (activeCategory.value) {
    list = list.filter((item) => item.category === activeCategory.value);
  }
  const keyword = searchKeyword.value.trim().toLowerCase();
  if (keyword) {
    list = list.filter(
      (item) =>
        item.config_key.toLowerCase().includes(keyword) ||
        (item.description ?? '').toLowerCase().includes(keyword),
    );
  }
  return list;
});

/** 视图分组：全部视图按分类分组，分类视图为单个无表头分组 */
const sections = computed<ConfigSection[]>(() => {
  if (activeCategory.value) {
    return [
      {
        category: activeCategory.value,
        label: categoryLabel(activeCategory.value),
        items: filteredData.value,
        showHeader: false,
      },
    ];
  }
  const groups: ConfigSection[] = [];
  const groupMap = new Map<string, ConfigSection>();
  for (const item of filteredData.value) {
    const category = item.category || '';
    let group = groupMap.get(category);
    if (!group) {
      group = { category, label: categoryLabel(category), items: [], showHeader: true };
      groupMap.set(category, group);
      groups.push(group);
    }
    group.items.push(item);
  }
  return groups;
});

/** 按值类型初始化内联编辑值（类型口径：STRING / NUMBER / BOOLEAN / JSON） */
function initEditValue(item: ConfigRes): void {
  if (editorKind(item.config_type) === 'boolean') {
    booleanValueMap[item.id] = item.config_value === 'true' || item.config_value === '1';
  } else if (editorKind(item.config_type) === 'number') {
    numberValueMap[item.id] = Number(item.config_value);
  } else {
    textValueMap[item.id] = item.config_value || '';
  }
}

async function loadData(): Promise<void> {
  try {
    const data = await pageConfigs({ page: 1, page_size: FULL_PAGE_SIZE });
    allData.value = data.list;
    for (const item of allData.value) {
      initEditValue(item);
    }
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '参数列表加载失败');
  }
}

/** 将内联编辑值按值类型序列化为后端所需的字符串 */
function serializeEditValue(item: ConfigRes): string {
  if (editorKind(item.config_type) === 'boolean') {
    return booleanValueMap[item.id] ? 'true' : 'false';
  }
  if (editorKind(item.config_type) === 'number') {
    const value = numberValueMap[item.id];
    return Number.isFinite(value) ? String(value) : '0';
  }
  return textValueMap[item.id] ?? '';
}

async function handleSave(item: ConfigRes): Promise<void> {
  savingMap[item.id] = true;
  try {
    const configValue = serializeEditValue(item);
    // 内联保存只改值，其余字段原样回传，避免误清空
    await updateConfig(item.id, {
      config_key: item.config_key,
      config_value: configValue,
      config_type: item.config_type,
      description: item.description,
      category: item.category,
      sort_order: item.sort_order,
      config_level: item.config_level,
      is_modifiable: item.is_modifiable,
      is_encrypted: item.is_encrypted,
      is_restart: item.is_restart,
      status: item.status,
      ref_value: item.ref_value,
      select_value: item.select_value,
    });
    item.config_value = configValue;
    void MessagePlugin.success('保存成功');
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    savingMap[item.id] = false;
  }
}

function confirmRemove(row: ConfigRes): void {
  const dialog = DialogPlugin.confirm({
    header: '删除参数',
    body: `确认删除参数「${row.config_key}」？`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteConfig(row.id);
          void MessagePlugin.success('参数已删除');
          await loadData();
        } catch (error) {
          void MessagePlugin.error(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

/**
 * 刷新运行时配置。
 *
 * 调用后端 `POST /api/support/config/reload`：重新装载进程内参数快照并广播给
 * 订阅者（如 IP 归属地解析开关）。参数的增删改本身已自动完成刷新，本按钮用于
 * 多实例部署时让本实例立即跟随其它实例的变更，或直改数据库后强制重新读取。
 */
async function handleRefreshCache(): Promise<void> {
  reloadLoading.value = true;
  try {
    const keys = await reloadRuntimeConfig();
    void MessagePlugin.success(`运行时配置已重载，共 ${keys.length} 项`);
    await loadData();
  } finally {
    reloadLoading.value = false;
  }
}

const formVisible = ref(false);
const isEdit = ref(false);
const submitting = ref(false);
const formRef = ref<FormInstanceFunctions | null>(null);
const form = reactive<ConfigForm>({
  id: '',
  config_key: '',
  config_value: '',
  config_type: 'STRING',
  description: '',
  category: '',
  sort_order: 0,
  config_level: 0,
  is_modifiable: true,
  is_encrypted: false,
  is_restart: false,
  status: 1,
  ref_value: '',
  select_value: '',
});
const formRules: FormRules<ConfigForm> = {
  config_key: [{ required: true, message: '请输入配置键' }],
  config_type: [{ required: true, message: '请选择值类型' }],
};

function openCreate(): void {
  isEdit.value = false;
  Object.assign(form, {
    id: '',
    config_key: '',
    config_value: '',
    config_type: 'STRING',
    description: '',
    category: '',
    sort_order: 0,
    config_level: 0,
    is_modifiable: true,
    is_encrypted: false,
    is_restart: false,
    status: 1,
    ref_value: '',
    select_value: '',
  });
  formVisible.value = true;
}

async function openEdit(row: ConfigRes): Promise<void> {
  try {
    const detail = await getConfig(row.id);
    isEdit.value = true;
    Object.assign(form, {
      id: detail.id,
      config_key: detail.config_key,
      config_value: detail.config_value,
      config_type: detail.config_type,
      description: detail.description,
      category: detail.category,
      sort_order: detail.sort_order,
      config_level: detail.config_level,
      is_modifiable: detail.is_modifiable,
      is_encrypted: detail.is_encrypted,
      is_restart: detail.is_restart,
      status: detail.status,
      ref_value: detail.ref_value,
      select_value: detail.select_value,
    });
    formVisible.value = true;
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '参数详情加载失败');
  }
}

async function submitForm(): Promise<void> {
  const valid = await formRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload = {
    config_key: form.config_key,
    config_value: form.config_value,
    config_type: form.config_type,
    description: form.description,
    category: form.category,
    sort_order: form.sort_order,
    config_level: form.config_level,
    is_modifiable: form.is_modifiable,
    is_encrypted: form.is_encrypted,
    is_restart: form.is_restart,
    status: form.status,
    ref_value: form.ref_value,
    select_value: form.select_value,
  };
  submitting.value = true;
  try {
    if (isEdit.value) {
      await updateConfig(form.id, payload);
      void MessagePlugin.success('参数已更新');
    } else {
      await createConfig(payload);
      void MessagePlugin.success('参数已创建');
    }
    formVisible.value = false;
    await loadData();
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    submitting.value = false;
  }
}

/** 预加载分类字典；失败时仅回退展示原始编码，不影响列表 */
async function preloadCategoryDict(): Promise<void> {
  try {
    await dictStore.loadItems(DICT_CONFIG_CATEGORY);
  } catch {
    // 忽略字典加载失败，categoryLabel 会自动回退为原始编码
  }
}

onMounted(() => {
  void preloadCategoryDict();
  void loadData();
});
</script>

<style scoped lang="less">
.config-page {
  padding: 16px;
}

.tab-card {
  :deep(.t-tabs__nav-wrap) {
    padding-left: 8px;
  }
}

.list-card {
  margin-top: 16px;
}

.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.list-toolbar-count {
  font-size: 13px;
  color: var(--td-text-color-secondary);
}

/* 分组样式 */
.config-group {
  margin-bottom: 24px;

  &:last-child {
    margin-bottom: 0;
  }
}

.config-group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  margin-bottom: 4px;
  background: var(--td-bg-color-secondarycontainer);
  border-radius: 6px;
  position: sticky;
  top: 0;
  z-index: 1;

  .t-icon {
    color: var(--td-brand-color);
  }
}

.config-group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--td-text-color-primary);
}

.config-group-count {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  margin-left: auto;
}

.config-group-body {
  padding: 0 4px;
}

.config-item {
  border-bottom: 1px solid var(--td-component-stroke);
  padding: 14px 10px;
  transition: background 0.2s;

  &:hover {
    background: var(--td-bg-color-container-hover);
  }

  &:last-child {
    border-bottom: none;
  }
}

.config-header {
  margin-bottom: 10px;
}

.config-title-area {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  min-width: 0;
}

.config-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--td-text-color-primary);
}

.config-key-sub {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  font-family: monospace;
}

.config-body {
  display: flex;
  flex-direction: column;
}

.config-value-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.config-value-area {
  width: 60%;
}

.config-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: auto;
  padding-top: 2px;
}
</style>
