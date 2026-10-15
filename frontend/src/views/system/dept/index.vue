<template>
  <div class="dept-page">
    <!-- 筛选栏 -->
    <t-card :bordered="false" class="filter-card">
      <t-row :gutter="16">
        <t-col :span="2.4">
          <t-input v-model="query.dept_name" placeholder="部门名称" clearable @enter="handleSearch" />
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

    <!-- 表格 -->
    <t-card :bordered="false" class="table-card">
      <div class="toolbar">
        <t-space>
          <t-button v-privilege="['system:dept:create']" theme="primary" @click="openCreate(undefined)">
            <template #icon><t-icon name="add" /></template>
            新增部门
          </t-button>
          <t-button theme="default" variant="outline" @click="toggleExpandAll">
            {{ allExpanded ? '收起全部' : '展开全部' }}
          </t-button>
          <t-button theme="default" @click="loadTree">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="toolbar__count">共 {{ totalNodes }} 个部门</span>
      </div>

      <t-enhanced-table
        row-key="id"
        :data="deptTree"
        :columns="columns"
        :loading="loading"
        :tree="treeConfig"
        :hover="true"
        :stripe="true"
        v-model:expanded-tree-nodes="expandedNodes"
      >
        <template #dept_name="{ row }">
          <span>{{ row.dept_name }}</span>
        </template>
        <template #leader="{ row }">
          <span>{{ row.leader_name || row.leader_id || '-' }}</span>
        </template>
        <template #status="{ row }">
          <t-tag :theme="statusTheme(row.status)" variant="light">{{ statusLabel(row.status) }}</t-tag>
        </template>
        <template #op="{ row }">
          <t-space size="small">
            <t-link v-privilege="['system:dept:create']" theme="primary" @click="openCreate(row)">
              新增下级
            </t-link>
            <t-link v-privilege="['system:dept:update']" theme="primary" @click="openEdit(row)">
              编辑
            </t-link>
            <t-link v-privilege="['system:dept:delete']" theme="danger" @click="confirmRemove(row)">
              删除
            </t-link>
          </t-space>
        </template>
      </t-enhanced-table>
    </t-card>

    <t-dialog
      v-model:visible="formVisible"
      :header="isEdit ? '编辑部门' : '新增部门'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: submitting }"
      width="540px"
      destroy-on-close
      @confirm="submitForm"
    >
      <t-form ref="formRef" :data="form" :rules="formRules" label-width="90px">
        <t-form-item label="上级部门" name="parent_id">
          <dept-tree-select v-model="form.parent_id" placeholder="不选则为顶级部门" @error="showError" />
        </t-form-item>
        <t-form-item label="部门名称" name="dept_name">
          <t-input v-model="form.dept_name" placeholder="请输入部门名称" />
        </t-form-item>
        <t-form-item label="部门编码" name="dept_code">
          <t-input v-model="form.dept_code" placeholder="请输入部门编码" />
        </t-form-item>
        <t-form-item label="负责人" name="leader_id">
          <t-select
            v-model="form.leader_id"
            placeholder="请选择负责人"
            clearable
            filterable
            :options="leaderOptions"
          />
        </t-form-item>
        <t-form-item label="联系电话" name="phone">
          <t-input v-model="form.phone" placeholder="请输入联系电话" />
        </t-form-item>
        <t-form-item label="邮箱" name="email">
          <t-input v-model="form.email" placeholder="请输入邮箱" />
        </t-form-item>
        <t-form-item label="排序" name="sort_order">
          <t-input-number v-model="form.sort_order" :min="0" :max="9999" theme="normal" />
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
  type PrimaryTableCol,
} from 'tdesign-vue-next';
import DeptTreeSelect from '@/components/DeptTreeSelect.vue';
import {
  createDept,
  deleteDept,
  getDept,
  getDeptTree,
  updateDept,
  type DeptNode,
  type DeptSaveParams,
} from '@/api/system/dept-api';
import type { StatusValue, UserRes } from '@/api/system/user-api';
import { pageUsers } from '@/api/system/user-api';
import { ApiError } from '@/api/request';

/** 查询条件 */
interface DeptQuery {
  /** 部门名称 */
  dept_name: string;
  /** 状态 */
  status: StatusValue | undefined;
}

/** 新增/编辑表单 */
interface DeptForm {
  /** 部门 id，新增时为空 */
  id: string;
  /** 部门编码 */
  dept_code: string;
  /** 部门名称 */
  dept_name: string;
  /** 上级部门 id */
  parent_id: string | undefined;
  /** 负责人用户 id */
  leader_id: string;
  /** 联系电话 */
  phone: string;
  /** 邮箱 */
  email: string;
  /** 状态 */
  status: StatusValue;
  /** 排序值 */
  sort_order: number;
}

const query = reactive<DeptQuery>({ dept_name: '', status: undefined });
const deptTree = ref<DeptNode[]>([]);
const loading = ref(false);
const expandedNodes = ref<Array<string | number>>([]);

const columns: PrimaryTableCol[] = [
  { colKey: 'dept_name', title: '部门名称', width: 240, cell: 'dept_name' },
  { colKey: 'dept_code', title: '部门编码', width: 180 },
  { colKey: 'leader_id', title: '负责人', width: 180, cell: 'leader' },
  { colKey: 'phone', title: '联系电话', width: 150 },
  { colKey: 'email', title: '邮箱', ellipsis: true },
  { colKey: 'sort_order', title: '排序', width: 80 },
  { colKey: 'status', title: '状态', width: 90, cell: 'status' },
  { colKey: 'op', title: '操作', width: 200, fixed: 'right' },
];

/** 树形表格配置：子节点字段为 children，树节点渲染在第 0 列 */
const treeConfig = { childrenKey: 'children', treeNodeColumnIndex: 0 };

const formVisible = ref(false);
const isEdit = ref(false);
const submitting = ref(false);
const formRef = ref<FormInstanceFunctions | null>(null);
const form = reactive<DeptForm>({
  id: '',
  dept_code: '',
  dept_name: '',
  parent_id: undefined,
  leader_id: '',
  phone: '',
  email: '',
  status: 1,
  sort_order: 0,
});
const formRules: FormRules<DeptForm> = {
  dept_code: [{ required: true, message: '请输入部门编码' }],
  dept_name: [{ required: true, message: '请输入部门名称' }],
  email: [{ email: true, message: '邮箱格式不正确' }],
};

/** 负责人下拉候选（全部启用用户，real_name 优先展示） */
const leaderOptions = ref<{ label: string; value: string }[]>([]);

/** 加载负责人下拉候选：拉取一次全量用户，供新增 / 编辑选择 */
async function loadLeaderOptions(): Promise<void> {
  try {
    const data = await pageUsers({ page: 1, page_size: 200 });
    leaderOptions.value = data.list.map((user: UserRes) => ({
      label: user.real_name || user.username,
      value: user.id,
    }));
  } catch {
    // 候选拉取失败不阻断主流程：负责人为可选字段
    leaderOptions.value = [];
  }
}

/** 部门总数，用于工具栏统计 */
const totalNodes = computed(() => countNodes(deptTree.value));

/** 是否已全部展开 */
const allExpanded = computed(() => {
  const ids = collectIds(deptTree.value);
  return ids.length > 0 && ids.every((id) => expandedNodes.value.includes(id));
});

/** 统计部门数量 */
function countNodes(nodes: DeptNode[]): number {
  return nodes.reduce((total, node) => total + 1 + countNodes(node.children ?? []), 0);
}

/** 收集所有部门 id，用于展开全部 */
function collectIds(nodes: DeptNode[]): string[] {
  const ids: string[] = [];
  for (const node of nodes) {
    ids.push(node.id);
    ids.push(...collectIds(node.children ?? []));
  }
  return ids;
}

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

async function loadTree(): Promise<void> {
  loading.value = true;
  try {
    deptTree.value = await getDeptTree({
      dept_name: query.dept_name || undefined,
      status: query.status,
    });
    expandedNodes.value = collectIds(deptTree.value);
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '部门树加载失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch(): void {
  void loadTree();
}

function handleReset(): void {
  query.dept_name = '';
  query.status = undefined;
  void loadTree();
}

function toggleExpandAll(): void {
  expandedNodes.value = allExpanded.value ? [] : collectIds(deptTree.value);
}

function openCreate(parent: DeptNode | undefined): void {
  isEdit.value = false;
  Object.assign(form, {
    id: '',
    dept_code: '',
    dept_name: '',
    parent_id: parent?.id,
    leader_id: '',
    phone: '',
    email: '',
    status: 1,
    sort_order: 0,
  });
  formVisible.value = true;
}

async function openEdit(row: DeptNode): Promise<void> {
  try {
    const detail = await getDept(row.id);
    isEdit.value = true;
    Object.assign(form, {
      id: detail.id,
      dept_code: detail.dept_code,
      dept_name: detail.dept_name,
      parent_id: detail.parent_id ?? undefined,
      leader_id: detail.leader_id ?? '',
      phone: detail.phone,
      email: detail.email,
      status: detail.status,
      sort_order: detail.sort_order,
    });
    formVisible.value = true;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '部门详情加载失败');
  }
}

async function submitForm(): Promise<void> {
  const valid = await formRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload: DeptSaveParams = {
    dept_code: form.dept_code,
    dept_name: form.dept_name,
    parent_id: form.parent_id || undefined,
    leader_id: form.leader_id || undefined,
    phone: form.phone || undefined,
    email: form.email || undefined,
    status: form.status,
    sort_order: form.sort_order,
  };
  submitting.value = true;
  try {
    if (isEdit.value) {
      await updateDept(form.id, payload);
      void MessagePlugin.success('部门已更新');
    } else {
      await createDept(payload);
      void MessagePlugin.success('部门已创建');
    }
    formVisible.value = false;
    await loadTree();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    submitting.value = false;
  }
}

function confirmRemove(row: DeptNode): void {
  const dialog = DialogPlugin.confirm({
    header: '删除部门',
    body: `确认删除部门「${row.dept_name}」？存在下级部门时需先删除下级。`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteDept(row.id);
          void MessagePlugin.success('部门已删除');
          await loadTree();
        } catch (error) {
          showError(error instanceof ApiError ? error.message : '删除失败');
        }
      })();
    },
  });
}

onMounted(() => {
  void loadTree();
  void loadLeaderOptions();
});
</script>

<style lang="less" scoped>
.dept-page {
  padding: 16px;

  .filter-card {
    margin-bottom: 12px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;

    &__count {
      font-size: 13px;
      color: var(--td-text-color-secondary);
    }
  }
}
</style>