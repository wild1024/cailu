<template>
  <div class="menu-page">
    <!-- 筛选栏 -->
    <t-card :bordered="false" class="filter-card">
      <t-row :gutter="16">
        <t-col :span="2.4">
          <t-input v-model="query.name" placeholder="菜单名称" clearable @enter="handleSearch" />
        </t-col>
        <t-col :span="2.4">
          <t-input v-model="query.module_code" placeholder="模块编码" clearable @enter="handleSearch" />
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
          <t-button v-privilege="['system:menu:create']" theme="primary" @click="openCreate(undefined)">
            <template #icon><t-icon name="add" /></template>
            新增菜单
          </t-button>
          <t-button theme="default" variant="outline" @click="toggleExpandAll">
            {{ allExpanded ? '收起全部' : '展开全部' }}
          </t-button>
          <t-button theme="default" @click="loadTree">
            <template #icon><t-icon name="refresh" /></template>
            刷新
          </t-button>
        </t-space>
        <span class="toolbar__count">共 {{ totalNodes }} 个节点</span>
      </div>

      <t-enhanced-table
        row-key="id"
        :data="menuTree"
        :columns="columns"
        :loading="loading"
        :tree="treeConfig"
        :hover="true"
        :stripe="true"
        v-model:expanded-tree-nodes="expandedNodes"
      >
        <template #title="{ row }">
          <t-space size="small">
            <t-icon v-if="row.icon" :name="row.icon" />
            <span>{{ row.title }}</span>
          </t-space>
        </template>
        <template #menu_type="{ row }">
          <t-tag :theme="menuTypeTheme(row.menu_type)" variant="light">
            {{ menuTypeLabel(row.menu_type) }}
          </t-tag>
        </template>
        <template #status="{ row }">
          <t-tag :theme="statusTheme(row.status)" variant="light">{{ statusLabel(row.status) }}</t-tag>
        </template>
        <template #op="{ row }">
          <t-space size="small">
            <t-link
              v-if="row.menu_type !== 3"
              v-privilege="['system:menu:create']"
              theme="primary"
              @click="openCreate(row)"
            >
              新增下级
            </t-link>
            <t-link v-privilege="['system:menu:update']" theme="primary" @click="openEdit(row)">
              编辑
            </t-link>
            <t-link v-privilege="['system:menu:delete']" theme="danger" @click="confirmRemove(row)">
              删除
            </t-link>
          </t-space>
        </template>
      </t-enhanced-table>
    </t-card>

    <t-dialog
      v-model:visible="formVisible"
      :header="isEdit ? '编辑菜单' : '新增菜单'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: submitting }"
      width="620px"
      destroy-on-close
      @confirm="submitForm"
    >
      <t-form ref="formRef" :data="form" :rules="formRules" label-width="90px">
        <t-form-item label="菜单类型" name="menu_type">
          <t-radio-group v-model="form.menu_type">
            <t-radio :value="1">目录</t-radio>
            <t-radio :value="2">菜单</t-radio>
            <t-radio :value="3">按钮</t-radio>
          </t-radio-group>
        </t-form-item>
        <t-form-item label="上级菜单" name="parent_id">
          <t-tree-select
            :value="form.parent_id || undefined"
            :data="parentOptions"
            :tree-props="{ keys: menuTreeKeys }"
            placeholder="不选则为顶级菜单"
            clearable
            filterable
            @change="onParentChange"
          />
        </t-form-item>
        <t-form-item label="菜单标题" name="title">
          <t-input v-model="form.title" placeholder="请输入菜单标题" />
        </t-form-item>
        <t-form-item label="菜单标识" name="name">
          <t-input v-model="form.name" placeholder="请输入菜单唯一标识，如 system:user" />
        </t-form-item>
        <t-form-item v-if="form.menu_type !== 3" label="路由路径" name="path">
          <t-input v-model="form.path" placeholder="请输入路由路径，如 /system/user" />
        </t-form-item>
        <t-form-item v-if="form.menu_type === 2" label="组件路径" name="component">
          <t-input v-model="form.component" placeholder="请输入组件路径，如 system/user/index" />
        </t-form-item>
        <t-form-item label="权限码" name="auth_code">
          <t-input v-model="form.auth_code" placeholder="请输入权限码，如 system:user:list" />
        </t-form-item>
        <t-form-item label="图标" name="icon">
          <IconSelect v-model="form.icon" placeholder="请选择或输入图标名，如 user" />
        </t-form-item>
        <t-form-item v-if="form.menu_type !== 3" label="模块编码" name="module_code">
          <t-input v-model="form.module_code" placeholder="请输入模块编码，如 system" />
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
        <t-form-item v-if="form.menu_type !== 3" label="显示导航" name="is_visible">
          <t-switch v-model="form.is_visible" />
        </t-form-item>
        <t-form-item v-if="form.menu_type === 2" label="是否外链" name="is_frame">
          <t-switch v-model="form.is_frame" />
        </t-form-item>
        <t-form-item v-if="form.menu_type === 2" label="页面缓存" name="is_cache">
          <t-switch v-model="form.is_cache" />
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
  createMenu,
  deleteMenu,
  getMenu,
  getMenuTree,
  updateMenu,
  type MenuSaveParams,
} from '@/api/system/menu-api';
import IconSelect from '@/components/IconSelect/index.vue';
import type { MenuNode, MenuType } from '@/api/system/auth-api';
import type { StatusValue } from '@/api/system/user-api';
import { ApiError } from '@/api/request';

/** 查询条件 */
interface MenuQuery {
  /** 菜单名称 */
  name: string;
  /** 模块编码 */
  module_code: string;
  /** 状态 */
  status: StatusValue | undefined;
}

/** 新增/编辑表单 */
interface MenuForm {
  /** 菜单 id，新增时为空 */
  id: string;
  /** 菜单名称 */
  name: string;
  /** 菜单类型 */
  menu_type: MenuType;
  /** 权限码 */
  auth_code: string;
  /** 路由路径 */
  path: string;
  /** 上级菜单 id */
  parent_id: string | null;
  /** 组件路径 */
  component: string;
  /** 排序值 */
  sort_order: number;
  /** 状态 */
  status: StatusValue;
  /** 图标 */
  icon: string;
  /** 菜单标题 */
  title: string;
  /** 是否外链 */
  is_frame: boolean;
  /** 是否缓存 */
  is_cache: boolean;
  /** 是否显示 */
  is_visible: boolean;
  /** 模块编码 */
  module_code: string;
}

const query = reactive<MenuQuery>({ name: '', module_code: '', status: undefined });
const menuTree = ref<MenuNode[]>([]);
const loading = ref(false);
const expandedNodes = ref<Array<string | number>>([]);

const columns: PrimaryTableCol[] = [
  { colKey: 'title', title: '菜单标题', width: 220, cell: 'title' },
  { colKey: 'menu_type', title: '类型', width: 90, cell: 'menu_type' },
  { colKey: 'auth_code', title: '权限码', width: 200 },
  { colKey: 'path', title: '路由路径', width: 180 },
  { colKey: 'component', title: '组件路径', ellipsis: true },
  { colKey: 'sort_order', title: '排序', width: 80 },
  { colKey: 'status', title: '状态', width: 90, cell: 'status' },
  { colKey: 'op', title: '操作', width: 210, fixed: 'right' },
];

/** 树形表格配置：子节点字段为 children，树节点渲染在第 0 列 */
const treeConfig = { childrenKey: 'children', treeNodeColumnIndex: 0 };

/** 菜单树字段映射 */
const menuTreeKeys = { value: 'id', label: 'title', children: 'children' };

/** 上级菜单候选项：排除按钮节点 */
const parentOptions = computed<MenuNode[]>(() => filterParentOptions(menuTree.value));

const formVisible = ref(false);
const isEdit = ref(false);
const submitting = ref(false);
const formRef = ref<FormInstanceFunctions | null>(null);
const form = reactive<MenuForm>({
  id: '',
  name: '',
  menu_type: 2,
  auth_code: '',
  path: '',
  parent_id: null,
  component: '',
  sort_order: 0,
  status: 1,
  icon: '',
  title: '',
  is_frame: false,
  is_cache: false,
  is_visible: true,
  module_code: '',
});
const formRules: FormRules<MenuForm> = {
  title: [{ required: true, message: '请输入菜单标题' }],
  name: [{ required: true, message: '请输入菜单标识' }],
  menu_type: [{ required: true, message: '请选择菜单类型' }],
};

/** 节点总数，用于工具栏统计 */
const totalNodes = computed(() => countNodes(menuTree.value));

/** 是否已全部展开 */
const allExpanded = computed(() => {
  const ids = collectIds(menuTree.value);
  return ids.length > 0 && ids.every((id) => expandedNodes.value.includes(id));
});

/** 统计树节点数量 */
function countNodes(nodes: MenuNode[]): number {
  return nodes.reduce((total, node) => total + 1 + countNodes(node.children ?? []), 0);
}

/** 收集所有节点 id，用于展开全部 */
function collectIds(nodes: MenuNode[]): string[] {
  const ids: string[] = [];
  for (const node of nodes) {
    ids.push(node.id);
    ids.push(...collectIds(node.children ?? []));
  }
  return ids;
}

/** 过滤出可作为上级菜单的节点（按钮不能有下级） */
function filterParentOptions(nodes: MenuNode[]): MenuNode[] {
  return nodes
    .filter((node) => node.menu_type !== 3)
    .map((node) => ({ ...node, children: filterParentOptions(node.children ?? []) }));
}

/** 菜单类型标签主题色 */
function menuTypeTheme(menuType: MenuType): 'primary' | 'success' | 'warning' {
  if (menuType === 1) {
    return 'primary';
  }
  return menuType === 2 ? 'success' : 'warning';
}

/** 菜单类型文案 */
function menuTypeLabel(menuType: MenuType): string {
  if (menuType === 1) {
    return '目录';
  }
  return menuType === 2 ? '菜单' : '按钮';
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
    menuTree.value = await getMenuTree({
      name: query.name || undefined,
      module_code: query.module_code || undefined,
      status: query.status,
    });
    expandedNodes.value = collectIds(menuTree.value);
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '菜单树加载失败');
  } finally {
    loading.value = false;
  }
}

function handleSearch(): void {
  void loadTree();
}

function handleReset(): void {
  query.name = '';
  query.module_code = '';
  query.status = undefined;
  void loadTree();
}

function toggleExpandAll(): void {
  expandedNodes.value = allExpanded.value ? [] : collectIds(menuTree.value);
}

/** 上级菜单变化：清空时置为 null 表示顶级菜单 */
function onParentChange(value: unknown): void {
  form.parent_id = typeof value === 'string' && value ? value : null;
}

function openCreate(parent: MenuNode | undefined): void {
  isEdit.value = false;
  Object.assign(form, {
    id: '',
    name: '',
    menu_type: parent ? 2 : 1,
    auth_code: '',
    path: '',
    parent_id: parent?.id ?? null,
    component: '',
    sort_order: 0,
    status: 1,
    icon: '',
    title: '',
    is_frame: false,
    is_cache: false,
    is_visible: true,
    module_code: '',
  });
  formVisible.value = true;
}

async function openEdit(row: MenuNode): Promise<void> {
  try {
    const detail = await getMenu(row.id);
    isEdit.value = true;
    Object.assign(form, {
      id: detail.id,
      name: detail.name,
      menu_type: detail.menu_type,
      auth_code: detail.auth_code,
      path: detail.path,
      parent_id: detail.parent_id,
      component: detail.component,
      sort_order: detail.sort_order,
      status: detail.status,
      icon: detail.icon,
      title: detail.title,
      is_frame: detail.is_frame,
      is_cache: detail.is_cache,
      is_visible: detail.is_visible,
      module_code: detail.module_code,
    });
    formVisible.value = true;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '菜单详情加载失败');
  }
}

async function submitForm(): Promise<void> {
  const valid = await formRef.value?.validate();
  if (valid !== true) {
    return;
  }
  const payload: MenuSaveParams = {
    name: form.name,
    menu_type: form.menu_type,
    auth_code: form.auth_code,
    path: form.path,
    parent_id: form.parent_id,
    component: form.component,
    sort_order: form.sort_order,
    status: form.status,
    icon: form.icon,
    title: form.title,
    is_frame: form.is_frame,
    is_cache: form.is_cache,
    is_visible: form.is_visible,
    module_code: form.module_code,
  };
  submitting.value = true;
  try {
    if (isEdit.value) {
      await updateMenu(form.id, payload);
      void MessagePlugin.success('菜单已更新');
    } else {
      await createMenu(payload);
      void MessagePlugin.success('菜单已创建');
    }
    formVisible.value = false;
    await loadTree();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    submitting.value = false;
  }
}

function confirmRemove(row: MenuNode): void {
  const dialog = DialogPlugin.confirm({
    header: '删除菜单',
    body: `确认删除菜单「${row.title}」？存在下级菜单时需先删除下级。`,
    theme: 'danger',
    confirmBtn: { content: '删除', theme: 'danger' },
    onConfirm: () => {
      dialog.destroy();
      void (async () => {
        try {
          await deleteMenu(row.id);
          void MessagePlugin.success('菜单已删除');
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
});
</script>

<style lang="less" scoped>
.menu-page {
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