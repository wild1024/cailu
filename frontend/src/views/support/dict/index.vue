<template>
  <div class="dict-page">
    <div class="dict-layout">
      <!-- 左侧：字典类型列表 -->
      <div class="dict-left">
        <t-card :bordered="false" class="dict-type-card">
          <div class="dict-type-toolbar">
            <span class="dict-type-title">字典类型</span>
            <t-button
              theme="primary"
              size="small"
              v-privilege="['support:dict:create']"
              @click="openCreateType"
            >
              <template #icon><t-icon name="add" /></template>
            </t-button>
          </div>
          <div class="dict-type-search">
            <t-input v-model="typeSearch" placeholder="搜索字典类型" clearable>
              <template #prefix-icon><t-icon name="search" /></template>
            </t-input>
          </div>
          <div class="dict-type-body">
            <t-loading v-if="typeLoading" :loading="true" size="small" text="加载中..." />
            <div
              v-for="item in filteredTypes"
              :key="item.id"
              class="dict-type-item"
              :class="{ active: selectedType?.id === item.id }"
              @click="selectType(item)"
            >
              <div class="dict-type-info">
                <div class="dict-type-name-line">
                  <span class="dict-type-name">{{ item.dict_name }}</span>
                  <t-tag :theme="statusTheme(item.status)" size="small" variant="light">
                    {{ statusLabel(item.status) }}
                  </t-tag>
                </div>
                <span class="dict-type-code">{{ item.dict_code }}</span>
              </div>
              <div class="dict-type-actions" @click.stop>
                <t-link
                  v-privilege="['support:dict:update']"
                  theme="primary"
                  @click="openEditType(item)"
                >
                  <t-icon name="edit" />
                </t-link>
                <t-link
                  v-privilege="['support:dict:delete']"
                  theme="danger"
                  @click="confirmRemoveType(item)"
                >
                  <t-icon name="delete" />
                </t-link>
              </div>
            </div>
            <div v-if="!typeLoading && types.length === 0" class="dict-type-empty">暂无字典类型</div>
          </div>
        </t-card>
      </div>

      <!-- 右侧：字典项表格 -->
      <div class="dict-right">
        <t-card v-if="selectedType" :bordered="false">
          <template #title>
            <span>字典项 — {{ selectedType.dict_name }} ({{ selectedType.dict_code }})</span>
          </template>
          <div class="dict-item-toolbar">
            <t-input v-model="itemSearch" placeholder="搜索字典项" clearable style="width: 220px">
              <template #prefix-icon><t-icon name="search" /></template>
            </t-input>
            <t-button
              theme="primary"
              size="small"
              v-privilege="['support:dict:create']"
              @click="openCreateItem"
            >
              <template #icon><t-icon name="add" /></template>
              新增
            </t-button>
          </div>
          <div class="dict-item-table-wrapper">
            <t-table
              row-key="id"
              :data="filteredItems"
              :columns="itemColumns"
              :loading="itemLoading"
              hover
              stripe
            >
              <template #is_default="{ row }">
                <t-tag v-if="row.is_default" theme="primary" variant="light">默认</t-tag>
                <span v-else>-</span>
              </template>
              <template #status="{ row }">
                <t-tag :theme="statusTheme(row.status)" variant="light">
                  {{ statusLabel(row.status) }}
                </t-tag>
              </template>
              <template #created_at="{ row }">{{ formatDateTime(row.created_at) }}</template>
              <template #op="{ row }">
                <t-space size="small">
                  <t-link
                    v-privilege="['support:dict:update']"
                    theme="primary"
                    @click="openEditItem(row)"
                  >
                    编辑
                  </t-link>
                  <t-link
                    v-privilege="['support:dict:delete']"
                    theme="danger"
                    @click="confirmRemoveItem(row)"
                  >
                    删除
                  </t-link>
                </t-space>
              </template>
            </t-table>
          </div>
        </t-card>
        <t-card v-else :bordered="false" class="dict-placeholder">
          <div class="dict-type-empty">请选择左侧字典类型查看字典项</div>
        </t-card>
      </div>
    </div>

    <!-- 字典类型新增/编辑弹窗 -->
    <t-dialog
      v-model:visible="typeFormVisible"
      :header="typeEditing ? '编辑字典类型' : '新增字典类型'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: typeSubmitting }"
      width="520px"
      destroy-on-close
      @confirm="submitType"
    >
      <t-form ref="typeFormRef" :data="typeForm" :rules="typeFormRules" label-width="90px">
        <t-form-item label="字典编码" name="dict_code">
          <t-input v-model="typeForm.dict_code" placeholder="请输入字典编码" :disabled="typeEditing" />
        </t-form-item>
        <t-form-item label="字典名称" name="dict_name">
          <t-input v-model="typeForm.dict_name" placeholder="请输入字典名称" />
        </t-form-item>
        <t-form-item label="描述" name="description">
          <t-input v-model="typeForm.description" placeholder="请输入描述" />
        </t-form-item>
        <t-form-item label="排序" name="sort_order">
          <t-input-number v-model="typeForm.sort_order" :min="0" :max="9999" theme="normal" />
        </t-form-item>
        <t-form-item label="状态" name="status">
          <t-radio-group v-model="typeForm.status">
            <t-radio :value="1">正常</t-radio>
            <t-radio :value="0">停用</t-radio>
          </t-radio-group>
        </t-form-item>
        <t-form-item label="备注" name="remark">
          <t-textarea v-model="typeForm.remark" placeholder="请输入备注" :maxlength="200" />
        </t-form-item>
      </t-form>
    </t-dialog>

    <!-- 字典项新增/编辑弹窗 -->
    <t-dialog
      v-model:visible="itemFormVisible"
      :header="itemEditing ? '编辑字典项' : '新增字典项'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: itemSubmitting }"
      width="520px"
      destroy-on-close
      @confirm="submitItem"
    >
      <t-form ref="itemFormRef" :data="itemForm" :rules="itemFormRules" label-width="90px">
        <t-form-item label="所属字典">
          <span>{{ selectedType?.dict_name }}</span>
        </t-form-item>
        <t-form-item label="字典标签" name="item_label">
          <t-input v-model="itemForm.item_label" placeholder="请输入字典标签" />
        </t-form-item>
        <t-form-item label="字典值" name="item_value">
          <t-input v-model="itemForm.item_value" placeholder="请输入字典值" />
        </t-form-item>
        <t-form-item label="描述" name="description">
          <t-input v-model="itemForm.description" placeholder="请输入描述" />
        </t-form-item>
        <t-form-item label="排序" name="sort_order">
          <t-input-number v-model="itemForm.sort_order" :min="0" :max="9999" theme="normal" />
        </t-form-item>
        <t-form-item label="默认项" name="is_default">
          <t-switch v-model="itemForm.is_default" />
        </t-form-item>
        <t-form-item label="状态" name="status">
          <t-radio-group v-model="itemForm.status">
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
  type PrimaryTableCol,
} from 'tdesign-vue-next';
import {
  createDictItem,
  createDictType,
  deleteDictItem,
  deleteDictType,
  getDictItem,
  getDictType,
  pageDictItems,
  pageDictTypes,
  updateDictItem,
  updateDictType,
  type DictItemRes,
  type DictTypeRes,
} from '@/api/support/dict-api';
import type { StatusValue } from '@/api/system/user-api';
import { ApiError } from '@/api/request';
import { useDictStore } from '@/stores/dict';
import { formatDateTime } from '@/utils/format';

/** 纯前端左右分栏过滤需全量数据，这里一次性拉取较大的分页 */
const FULL_PAGE_SIZE = 1000;

/** 字典类型表单 */
interface DictTypeForm {
  /** 类型 id，新增时为空 */
  id: string;
  /** 字典编码 */
  dict_code: string;
  /** 字典名称 */
  dict_name: string;
  /** 描述 */
  description: string;
  /** 排序值 */
  sort_order: number;
  /** 状态 */
  status: StatusValue;
  /** 备注 */
  remark: string;
}

/** 字典项表单 */
interface DictItemForm {
  /** 字典项 id，新增时为空 */
  id: string;
  /** 字典标签 */
  item_label: string;
  /** 字典值 */
  item_value: string;
  /** 描述 */
  description: string;
  /** 排序值 */
  sort_order: number;
  /** 是否默认项 */
  is_default: boolean;
  /** 状态 */
  status: StatusValue;
}

const dictStore = useDictStore();

const types = ref<DictTypeRes[]>([]);
const typeLoading = ref(false);
const typeSearch = ref('');

const selectedType = ref<DictTypeRes | null>(null);
const items = ref<DictItemRes[]>([]);
const itemLoading = ref(false);
const itemSearch = ref('');

const itemColumns: PrimaryTableCol[] = [
  { colKey: 'item_label', title: '字典标签', width: 140 },
  { colKey: 'item_value', title: '字典值', width: 120 },
  { colKey: 'is_default', title: '默认', width: 80, cell: 'is_default' },
  { colKey: 'sort_order', title: '排序', width: 80 },
  { colKey: 'status', title: '状态', width: 80, cell: 'status' },
  { colKey: 'created_at', title: '创建时间', width: 180, cell: 'created_at' },
  { colKey: 'op', title: '操作', width: 110 },
];

/** 左侧字典类型前端过滤（编码 / 名称 / 描述模糊匹配） */
const filteredTypes = computed<DictTypeRes[]>(() => {
  const keyword = typeSearch.value.trim().toLowerCase();
  if (!keyword) {
    return types.value;
  }
  return types.value.filter(
    (item) =>
      item.dict_code.toLowerCase().includes(keyword) ||
      item.dict_name.toLowerCase().includes(keyword) ||
      (item.description ?? '').toLowerCase().includes(keyword),
  );
});

/** 右侧字典项前端过滤（标签 / 值模糊匹配） */
const filteredItems = computed<DictItemRes[]>(() => {
  const keyword = itemSearch.value.trim().toLowerCase();
  if (!keyword) {
    return items.value;
  }
  return items.value.filter(
    (item) =>
      item.item_label.toLowerCase().includes(keyword) ||
      item.item_value.toLowerCase().includes(keyword),
  );
});

/** 状态标签主题色 */
function statusTheme(status: StatusValue): 'success' | 'danger' {
  return status === 1 ? 'success' : 'danger';
}

/** 状态文案 */
function statusLabel(status: StatusValue): string {
  return status === 1 ? '正常' : '停用';
}

/** 统一展示接口错误 */
function showError(message: string): void {
  void MessagePlugin.error(message);
}

async function loadTypes(): Promise<void> {
  typeLoading.value = true;
  try {
    const data = await pageDictTypes({ page: 1, page_size: FULL_PAGE_SIZE });
    types.value = data.list;
    // 保留原选中项；若其已被删除则回退到未选中状态
    if (selectedType.value && !types.value.some((item) => item.id === selectedType.value?.id)) {
      selectedType.value = null;
      items.value = [];
    }
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '字典类型加载失败');
  } finally {
    typeLoading.value = false;
  }
}

async function loadItems(): Promise<void> {
  if (!selectedType.value) {
    items.value = [];
    return;
  }
  itemLoading.value = true;
  try {
    const data = await pageDictItems({
      page: 1,
      page_size: FULL_PAGE_SIZE,
      dict_type_id: selectedType.value.id,
    });
    items.value = data.list;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '字典项加载失败');
  } finally {
    itemLoading.value = false;
  }
}

/** 选中左侧字典类型并加载右侧字典项 */
function selectType(row: DictTypeRes): void {
  selectedType.value = row;
  itemSearch.value = '';
  void loadItems();
}

const typeFormVisible = ref(false);
const typeEditing = ref(false);
const typeSubmitting = ref(false);
const typeFormRef = ref<FormInstanceFunctions | null>(null);
const typeForm = reactive<DictTypeForm>({
  id: '',
  dict_code: '',
  dict_name: '',
  description: '',
  sort_order: 0,
  status: 1,
  remark: '',
});
const typeFormRules: FormRules<DictTypeForm> = {
  dict_code: [{ required: true, message: '请输入字典编码' }],
  dict_name: [{ required: true, message: '请输入字典名称' }],
};

const itemFormVisible = ref(false);
const itemEditing = ref(false);
const itemSubmitting = ref(false);
const itemFormRef = ref<FormInstanceFunctions | null>(null);
const itemForm = reactive<DictItemForm>({
  id: '',
  item_label: '',
  item_value: '',
  description: '',
  sort_order: 0,
  is_default: false,
  status: 1,
});
const itemFormRules: FormRules<DictItemForm> = {
  item_label: [{ required: true, message: '请输入字典标签' }],
  item_value: [{ required: true, message: '请输入字典值' }],
};

function openCreateType(): void {
  typeEditing.value = false;
  Object.assign(typeForm, {
    id: '',
    dict_code: '',
    dict_name: '',
    description: '',
    sort_order: 0,
    status: 1,
    remark: '',
  });
  typeFormVisible.value = true;
}

async function openEditType(row: DictTypeRes): Promise<void> {
  try {
    const detail = await getDictType(row.id);
    typeEditing.value = true;
    Object.assign(typeForm, {
      id: detail.id,
      dict_code: detail.dict_code,
      dict_name: detail.dict_name,
      description: detail.description,
      sort_order: detail.sort_order,
      status: detail.status,
      remark: detail.remark,
    });
    typeFormVisible.value = true;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '字典类型详情加载失败');
  }
}

async function submitType(): Promise<void> {
  const valid = await typeFormRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload = {
    dict_code: typeForm.dict_code,
    dict_name: typeForm.dict_name,
    description: typeForm.description,
    sort_order: typeForm.sort_order,
    status: typeForm.status,
    remark: typeForm.remark,
  };
  typeSubmitting.value = true;
  try {
    if (typeEditing.value) {
      await updateDictType(typeForm.id, payload);
      void MessagePlugin.success('字典类型已更新');
    } else {
      await createDictType(payload);
      void MessagePlugin.success('字典类型已创建');
    }
    typeFormVisible.value = false;
    dictStore.clear();
    await loadTypes();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    typeSubmitting.value = false;
  }
}

function confirmRemoveType(row: DictTypeRes): void {
  const dialog = DialogPlugin.confirm({
    header: '删除字典类型',
    body: `确认删除字典类型「${row.dict_name}」？该字典下的字典项需先删除。`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteDictType(row.id);
          void MessagePlugin.success('字典类型已删除');
          if (selectedType.value?.id === row.id) {
            selectedType.value = null;
            items.value = [];
          }
          dictStore.clear(row.dict_code);
          await loadTypes();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

function openCreateItem(): void {
  itemEditing.value = false;
  Object.assign(itemForm, {
    id: '',
    item_label: '',
    item_value: '',
    description: '',
    sort_order: 0,
    is_default: false,
    status: 1,
  });
  itemFormVisible.value = true;
}

async function openEditItem(row: DictItemRes): Promise<void> {
  try {
    const detail = await getDictItem(row.id);
    itemEditing.value = true;
    Object.assign(itemForm, {
      id: detail.id,
      item_label: detail.item_label,
      item_value: detail.item_value,
      description: detail.description,
      sort_order: detail.sort_order,
      is_default: detail.is_default,
      status: detail.status,
    });
    itemFormVisible.value = true;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '字典项详情加载失败');
  }
}

async function submitItem(): Promise<void> {
  if (!selectedType.value) {
    return;
  }
  const valid = await itemFormRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload = {
    dict_type_id: selectedType.value.id,
    item_value: itemForm.item_value,
    item_label: itemForm.item_label,
    description: itemForm.description,
    sort_order: itemForm.sort_order,
    is_default: itemForm.is_default,
    status: itemForm.status,
  };
  itemSubmitting.value = true;
  try {
    if (itemEditing.value) {
      await updateDictItem(itemForm.id, payload);
      void MessagePlugin.success('字典项已更新');
    } else {
      await createDictItem(payload);
      void MessagePlugin.success('字典项已创建');
    }
    itemFormVisible.value = false;
    dictStore.clear(selectedType.value.dict_code);
    await loadItems();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    itemSubmitting.value = false;
  }
}

function confirmRemoveItem(row: DictItemRes): void {
  const dialog = DialogPlugin.confirm({
    header: '删除字典项',
    body: `确认删除字典项「${row.item_label}」？`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteDictItem(row.id);
          void MessagePlugin.success('字典项已删除');
          if (selectedType.value) {
            dictStore.clear(selectedType.value.dict_code);
          }
          await loadItems();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

onMounted(() => {
  void loadTypes();
});
</script>

<style scoped lang="less">
.dict-page {
  padding: 16px;
}

.dict-layout {
  display: flex;
  height: calc(100vh - 172px);
  overflow: hidden;
}

.dict-left {
  width: 280px;
  min-width: 280px;
  border-right: 1px solid var(--td-component-stroke);
  overflow: hidden;
}

.dict-type-card {
  height: 100%;
  display: flex;
  flex-direction: column;

  :deep(.t-card__body) {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }
}

.dict-type-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--td-component-stroke);

  .dict-type-title {
    font-size: 15px;
    font-weight: 600;
  }
}

.dict-type-search {
  padding: 8px 16px;
  border-bottom: 1px solid var(--td-component-stroke);
}

.dict-type-body {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.dict-type-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: all 0.2s;

  &:hover {
    background: var(--td-bg-color-container-hover);
  }

  &.active {
    background: var(--td-brand-color-light);
    border-left-color: var(--td-brand-color);
  }
}

.dict-type-info {
  flex: 1;
  min-width: 0;

  .dict-type-name-line {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dict-type-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--td-text-color-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dict-type-code {
    display: block;
    font-size: 12px;
    color: var(--td-text-color-secondary);
    margin-top: 2px;
  }
}

.dict-type-actions {
  display: none;
  gap: 4px;

  .dict-type-item:hover & {
    display: flex;
  }
}

.dict-type-empty {
  padding: 48px;
  text-align: center;
  color: var(--td-text-color-placeholder);
  font-size: 14px;
}

.dict-right {
  flex: 1;
  overflow: hidden;
  padding: 0 16px;
  display: flex;
  flex-direction: column;

  :deep(.t-card) {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  :deep(.t-card__body) {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
}

.dict-item-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.dict-item-table-wrapper {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.dict-placeholder {
  height: 100%;
}
</style>
