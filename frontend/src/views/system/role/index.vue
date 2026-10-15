<template>
  <div class="role-page">
    <div class="role-layout">
      <!-- 左侧：角色列表 -->
      <div class="role-left">
        <t-card :bordered="false" class="role-list-card">
          <div class="role-toolbar">
            <span class="role-title">角色列表</span>
            <t-button
              v-privilege="['system:role:create']"
              size="small"
              theme="primary"
              @click="openCreate"
            >
              <template #icon><t-icon name="add" /></template>
            </t-button>
          </div>
          <div class="role-list-body">
            <t-loading :loading="loading" size="small" text="加载中...">
              <t-tooltip
                v-for="role in roleList"
                :key="role.id"
                :content="role.description || '暂无描述'"
                placement="right"
                show-arrow
              >
                <div
                  class="role-item"
                  :class="{ active: selectedRoleId === role.id }"
                  @click="selectRole(role)"
                >
                  <div class="role-item-info">
                    <span class="role-item-name">{{ role.role_name }}</span>
                    <span class="role-item-code">{{ role.role_code }}</span>
                  </div>
                  <div class="role-item-actions" @click.stop>
                    <t-link
                      v-privilege="['system:role:update']"
                      size="small"
                      theme="primary"
                      @click="openEdit(role)"
                    >
                      <t-icon name="edit" />
                    </t-link>
                    <t-popconfirm content="确认删除该角色吗？" @confirm="handleDelete(role)">
                      <t-link
                        v-privilege="['system:role:delete']"
                        size="small"
                        theme="danger"
                      >
                        <t-icon name="delete" />
                      </t-link>
                    </t-popconfirm>
                  </div>
                </div>
              </t-tooltip>
              <div v-if="!loading && roleList.length === 0" class="role-empty">暂无角色</div>
            </t-loading>
          </div>
        </t-card>
      </div>

      <!-- 右侧：详情 -->
      <div class="role-right">
        <t-card v-if="selectedRole" :bordered="false" class="role-right-card">
          <t-tabs v-model="activeTab" @change="onTabChange">
            <t-tab-panel value="perm" label="角色功能权限">
              <t-loading :loading="permLoading" size="small">
                <div class="perm-tree-wrapper">
                  <div class="perm-controls">
                    <t-space>
                      <t-button theme="default" variant="outline" @click="togglePermExpand">
                        <template #icon>
                          <t-icon :name="permExpandAll ? 'chevron-up' : 'chevron-down'" />
                        </template>
                        {{ permExpandAll ? '折叠全部' : '展开全部' }}
                      </t-button>
                      <t-button theme="default" variant="outline" @click="togglePermSelectAll">
                        {{ permSelectAll ? '取消全选' : '全选' }}
                      </t-button>
                      <t-checkbox v-model="permCheckStrictly">父子联动</t-checkbox>
                    </t-space>
                  </div>
                  <t-enhanced-table
                    ref="permTableRef"
                    row-key="id"
                    :data="permTreeData"
                    :columns="permColumns"
                    :tree="permTreeConfig"
                    :selected-row-keys="selectedMenuIds"
                    v-model:expanded-tree-nodes="permExpandedKeys"
                    hover
                    stripe
                    table-layout="auto"
                    :max-height="permTableMaxHeight"
                    @expanded-tree-nodes-change="onPermExpandedChange"
                    @select-change="onPermSelectChange"
                  >
                    <template #icon="{ row }">
                      <t-icon v-if="row.icon" :name="row.icon" size="16px" />
                      <span v-else>-</span>
                    </template>
                    <template #menu_type="{ row }">
                      <t-tag :theme="menuTypeTheme(row.menu_type)" variant="light" size="small">
                        {{ menuTypeLabel(row.menu_type) }}
                      </t-tag>
                    </template>
                    <template #buttons="{ row }">
                      <t-checkbox-group
                        v-if="row.buttons && row.buttons.length > 0"
                        :value="checkedIds"
                        class="perm-button-group"
                        @change="(val: Array<string | number | boolean>) => onPermButtonsChange(row, val)"
                      >
                        <t-checkbox v-for="btn in row.buttons" :key="btn.id" :value="btn.id">
                          {{ btn.title || btn.name }}
                        </t-checkbox>
                      </t-checkbox-group>
                    </template>
                  </t-enhanced-table>
                </div>
                <div v-if="permTreeData.length === 0 && !permLoading" class="perm-empty">
                  暂无菜单数据
                </div>
                <div class="perm-footer">
                  <t-button
                    v-privilege="['system:role:assign']"
                    theme="primary"
                    :loading="savePermLoading"
                    @click="handleSavePerms"
                  >
                    保存权限
                  </t-button>
                </div>
              </t-loading>
            </t-tab-panel>
            <t-tab-panel value="users" label="角色用户列表">
              <t-table
                row-key="id"
                :data="roleUsers"
                :columns="userColumns"
                :loading="usersLoading"
                :pagination="usersPagination"
                :max-height="usersTableMaxHeight"
                hover
                stripe
                @page-change="handleUsersPageChange"
              >
                <template #roles="{ row }">
                  <t-space size="small">
                    <t-tag
                      v-for="name in row.roles"
                      :key="name"
                      theme="primary"
                      variant="light"
                      size="small"
                    >
                      {{ name }}
                    </t-tag>
                    <span v-if="row.roles.length === 0">-</span>
                  </t-space>
                </template>
                <template #status="{ row }">
                  <t-tag :theme="row.status === 1 ? 'success' : 'danger'" variant="light" size="small">
                    {{ row.status === 1 ? '正常' : '停用' }}
                  </t-tag>
                </template>
              </t-table>
              <div v-if="!usersLoading && roleUsers.length === 0" class="users-placeholder">
                <t-empty description="暂无用户拥有此角色" />
              </div>
            </t-tab-panel>
          </t-tabs>
        </t-card>
        <t-card v-else :bordered="false" class="perm-placeholder">
          <div class="perm-empty">请选择左侧角色查看详情</div>
        </t-card>
      </div>
    </div>

    <!-- 新增/编辑角色 -->
    <t-dialog
      v-model:visible="formVisible"
      :header="formMode === 'create' ? '新增角色' : '编辑角色'"
      :confirm-btn="{ content: '确定', theme: 'primary', loading: submitting }"
      width="520px"
      destroy-on-close
      @confirm="handleFormSubmit"
    >
      <t-form ref="formRef" :data="formData" :rules="formRules" label-width="80px">
        <t-form-item v-if="formMode === 'create'" label="角色编码" name="role_code">
          <t-input v-model="formData.role_code" placeholder="请输入角色编码" />
        </t-form-item>
        <t-form-item label="角色名" name="role_name">
          <t-input v-model="formData.role_name" placeholder="请输入角色名" />
        </t-form-item>
        <t-form-item label="描述" name="description">
          <t-textarea
            v-model="formData.description"
            placeholder="请输入描述"
            :autosize="{ minRows: 2 }"
          />
        </t-form-item>
        <t-form-item label="排序" name="sort_order">
          <t-input-number v-model="formData.sort_order" :min="0" />
        </t-form-item>
        <t-form-item label="状态" name="status">
          <t-radio-group v-model="formData.status">
            <t-radio :value="1">正常</t-radio>
            <t-radio :value="0">禁用</t-radio>
          </t-radio-group>
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, reactive, ref } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import type {
  EnhancedTableInstanceFunctions,
  FormInstanceFunctions,
  FormRules,
  PrimaryTableCol,
  TdEnhancedTableProps,
} from 'tdesign-vue-next';
import {
  createRole,
  deleteRole,
  getRole,
  getRoleMenus,
  getRoleUsers,
  pageRoles,
  updateRole,
  updateRoleMenus,
  type RoleRes,
  type RoleSaveParams,
  type RoleUserRes,
} from '@/api/system/role-api';
import { getMenuTree } from '@/api/system/menu-api';
import type { MenuNode, MenuType } from '@/api/system/auth-api';
import type { StatusValue } from '@/api/system/user-api';
import { ApiError } from '@/api/request';

/// 权限树节点：按钮类型（menu_type=3）被分离到父节点的 buttons 数组中
interface PermRow {
  /** 节点 ID */
  id: string;
  /** 显示名（title 优先，回退 name） */
  name: string;
  /** 图标名 */
  icon: string;
  /** 菜单类型：1=目录 2=菜单 3=按钮（本结构不含按钮节点） */
  menu_type: MenuType;
  /** 子菜单节点（不含按钮） */
  children: PermRow[];
  /** 关联的按钮节点列表 */
  buttons: MenuNode[];
}

/** 表格多选回调附加信息（仅取用所需字段） */
interface PermSelectOptions {
  /** 触发勾选的行主键 */
  currentRowKey?: string | number;
  /** 操作类型：勾选 / 取消勾选 */
  type?: 'check' | 'uncheck';
}

/** 新增/编辑角色表单 */
interface RoleForm {
  /** 角色编码（仅新增时可编辑） */
  role_code: string;
  /** 角色名称 */
  role_name: string;
  /** 角色描述 */
  description: string;
  /** 排序值 */
  sort_order: number;
  /** 状态：1=正常 0=禁用 */
  status: StatusValue;
}

// ===== 角色列表 =====
const loading = ref(false);
const roleList = ref<RoleRes[]>([]);
const selectedRoleId = ref('');
const selectedRole = ref<RoleRes | null>(null);

// ===== 权限 tab =====
const activeTab = ref<'perm' | 'users'>('perm');
const permTableRef = ref<EnhancedTableInstanceFunctions>();
const permTreeData = ref<PermRow[]>([]);
const checkedIds = ref<string[]>([]);
const selectedMenuIds = ref<Array<string | number>>([]);
const permExpandedKeys = ref<Array<string | number>>([]);
const permExpandAll = ref(true);
const permSelectAll = ref(false);
const permCheckStrictly = ref(true);
const permLoading = ref(false);
const savePermLoading = ref(false);
/// 权限表格动态高度：视口高度减去布局与页签占用
const permTableMaxHeight = ref(window.innerHeight - 380);
/// 标记「程序化更新选中集合」阶段，避免触发 select-change 的联动逻辑
const isPermUpdating = ref(false);
/// 权限树的树形展示配置
const permTreeConfig: TdEnhancedTableProps['tree'] = {
  childrenKey: 'children',
  treeNodeColumnIndex: 0,
  indent: 24,
  expandTreeNodeOnClick: true,
};

const permColumns: PrimaryTableCol[] = [
  { colKey: 'row-select', type: 'multiple', width: 50 },
  { colKey: 'name', title: '菜单名称', width: 200, ellipsis: true },
  { colKey: 'icon', title: '图标', width: 60, align: 'center', cell: 'icon' },
  { colKey: 'menu_type', title: '类型', width: 70, cell: 'menu_type' },
  { colKey: 'buttons', title: '按钮权限', cell: 'buttons' },
];

// ===== 角色用户列表 tab =====
const roleUsers = ref<RoleUserRes[]>([]);
const usersLoading = ref(false);
/// 角色用户表格动态高度：视口高度减去布局与页签占用
const usersTableMaxHeight = ref(window.innerHeight - 320);
const usersPagination = reactive({ current: 1, pageSize: 10, total: 0 });
const userColumns: PrimaryTableCol[] = [
  { colKey: 'username', title: '用户名称', width: 140 },
  { colKey: 'real_name', title: '用户昵称', width: 140 },
  { colKey: 'mobile', title: '手机号', width: 160 },
  { colKey: 'roles', title: '角色名称', cell: 'roles' },
  { colKey: 'status', title: '状态', width: 100, cell: 'status' },
];

// ===== 新增/编辑表单 =====
const formRef = ref<FormInstanceFunctions>();
const formVisible = ref(false);
const formMode = ref<'create' | 'edit'>('create');
const editingId = ref('');
const submitting = ref(false);
const formData = reactive<RoleForm>({
  role_code: '',
  role_name: '',
  description: '',
  sort_order: 0,
  status: 1,
});
const formRules: FormRules<RoleForm> = {
  role_code: [{ required: true, message: '请输入角色编码', type: 'error' }],
  role_name: [{ required: true, message: '请输入角色名', type: 'error' }],
};

/// 统一展示接口错误
function showError(message: string): void {
  void MessagePlugin.error(message);
}

/// 类型标签文案
function menuTypeLabel(menuType: MenuType): string {
  return menuType === 1 ? '目录' : menuType === 2 ? '菜单' : '按钮';
}

/// 类型标签主题色
function menuTypeTheme(menuType: MenuType): 'warning' | 'primary' | 'success' {
  return menuType === 1 ? 'warning' : menuType === 2 ? 'primary' : 'success';
}

/// 将菜单树转换为权限树：按钮类型（menu_type=3）分离到父节点的 buttons 数组
function transformPermTree(nodes: MenuNode[]): PermRow[] {
  return nodes.map((node) => {
    const childNodes = node.children ?? [];
    const buttonNodes = childNodes.filter((child) => child.menu_type === 3);
    const treeNodes = childNodes.filter((child) => child.menu_type !== 3);
    return {
      id: node.id,
      name: node.title || node.name,
      icon: node.icon,
      menu_type: node.menu_type,
      children: transformPermTree(treeNodes),
      buttons: buttonNodes,
    };
  });
}

/// 递归收集权限树中所有 ID（菜单 + 按钮）
function collectAllPermIds(nodes: PermRow[]): string[] {
  const ids: string[] = [];
  for (const node of nodes) {
    ids.push(node.id);
    for (const btn of node.buttons) {
      ids.push(btn.id);
    }
    if (node.children.length > 0) {
      ids.push(...collectAllPermIds(node.children));
    }
  }
  return ids;
}

/// 递归收集权限树中所有菜单 ID（不含按钮）
function collectAllMenuIds(nodes: PermRow[]): string[] {
  const ids: string[] = [];
  for (const node of nodes) {
    ids.push(node.id);
    if (node.children.length > 0) {
      ids.push(...collectAllMenuIds(node.children));
    }
  }
  return ids;
}

/// 收集指定节点的所有后代 ID（可选是否包含按钮）
function collectDescendantIds(row: PermRow, includeButtons: boolean): string[] {
  const ids: string[] = [];
  for (const child of row.children) {
    ids.push(child.id);
    if (includeButtons) {
      ids.push(...child.buttons.map((btn) => btn.id));
    }
    ids.push(...collectDescendantIds(child, includeButtons));
  }
  return ids;
}

/// 收集指定节点 ID 的所有祖先 ID；未找到返回 null
function collectAncestorIds(rowId: string, list?: PermRow[]): string[] | null {
  const searchList = list ?? permTreeData.value;
  for (const node of searchList) {
    if (node.id === rowId) return [];
    if (node.children.length > 0) {
      const path = collectAncestorIds(rowId, node.children);
      if (path !== null) {
        path.unshift(node.id);
        return path;
      }
    }
  }
  return null;
}

/// 在权限树中查找指定 ID 的节点
function findPermRow(id: string, list?: PermRow[]): PermRow | null {
  const searchList = list ?? permTreeData.value;
  for (const node of searchList) {
    if (node.id === id) return node;
    if (node.children.length > 0) {
      const found = findPermRow(id, node.children);
      if (found) return found;
    }
  }
  return null;
}

/// 根据当前选中集合更新「全选」状态
function updateSelectAllState(): void {
  const allIds = collectAllPermIds(permTreeData.value);
  permSelectAll.value = allIds.length > 0 && allIds.every((id) => checkedIds.value.includes(id));
}

/// 选中角色，加载权限数据
async function selectRole(role: RoleRes): Promise<void> {
  selectedRoleId.value = role.id;
  selectedRole.value = role;
  activeTab.value = 'perm';
  // 切换角色时清空上一角色的用户列表，避免残留
  roleUsers.value = [];
  usersPagination.current = 1;
  usersPagination.total = 0;
  await loadPermData();
}

/// 加载当前角色下的用户列表（分页）
async function loadRoleUsers(): Promise<void> {
  if (!selectedRoleId.value) return;
  usersLoading.value = true;
  try {
    const data = await getRoleUsers(selectedRoleId.value, {
      page: usersPagination.current,
      page_size: usersPagination.pageSize,
    });
    roleUsers.value = data.list;
    usersPagination.total = data.total;
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '角色用户加载失败');
    roleUsers.value = [];
    usersPagination.total = 0;
  } finally {
    usersLoading.value = false;
  }
}

/// 页签切换：进入「角色用户列表」时重置分页并加载
function onTabChange(value: string | number): void {
  activeTab.value = value === 'users' ? 'users' : 'perm';
  if (activeTab.value === 'users' && selectedRoleId.value) {
    usersPagination.current = 1;
    void loadRoleUsers();
  }
}

/// 角色用户表格分页变化
function handleUsersPageChange(pageInfo: { current: number; pageSize: number }): void {
  usersPagination.current = pageInfo.current;
  usersPagination.pageSize = pageInfo.pageSize;
  void loadRoleUsers();
}

/// 加载角色列表，默认选中第一个角色
async function loadList(): Promise<void> {
  loading.value = true;
  try {
    const data = await pageRoles({ page: 1, page_size: 1000 });
    roleList.value = data.list;
    if (!selectedRoleId.value && roleList.value.length > 0) {
      await selectRole(roleList.value[0]);
    } else if (selectedRoleId.value) {
      // 刷新当前选中角色的权限数据
      await loadPermData();
    }
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '角色列表加载失败');
  } finally {
    loading.value = false;
  }
}

/// 加载菜单树并转换为权限树
async function loadMenuTree(): Promise<void> {
  try {
    const tree = await getMenuTree();
    permTreeData.value = transformPermTree(tree);
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '菜单树加载失败');
    permTreeData.value = [];
  }
}

/// 加载当前角色已分配的菜单 + 按钮 ID
async function loadPermData(): Promise<void> {
  if (!selectedRoleId.value) return;
  permLoading.value = true;
  isPermUpdating.value = true;
  try {
    // 菜单树为空时先加载
    if (permTreeData.value.length === 0) {
      await loadMenuTree();
    }
    const menuIds = await getRoleMenus(selectedRoleId.value);
    const menuIdSet = new Set<string>(menuIds);
    checkedIds.value = menuIds;

    // 表格多选只接受菜单节点 ID（不含按钮）；若该行任一按钮已勾选，则该行也应显示为选中
    const menuSelected: Array<string | number> = [];
    function markRow(list: PermRow[]): void {
      for (const row of list) {
        if (menuIdSet.has(row.id) || row.buttons.some((btn) => menuIdSet.has(btn.id))) {
          menuSelected.push(row.id);
        }
        if (row.children.length > 0) {
          markRow(row.children);
        }
      }
    }
    markRow(permTreeData.value);
    selectedMenuIds.value = menuSelected;

    // 默认展开全部
    permExpandedKeys.value = collectAllMenuIds(permTreeData.value);
    permExpandAll.value = true;
    updateSelectAllState();
    await nextTick();
    permTableRef.value?.expandAll();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '权限数据加载失败');
    checkedIds.value = [];
    selectedMenuIds.value = [];
  } finally {
    permLoading.value = false;
    isPermUpdating.value = false;
  }
}

/// 多选回调：勾选/取消勾选父菜单行（含父子联动）
function onPermSelectChange(
  selectedKeys: Array<string | number>,
  options: PermSelectOptions,
): void {
  if (isPermUpdating.value) return;

  const currentRowKey = options.currentRowKey ? String(options.currentRowKey) : '';
  const actionType = options.type ?? 'check';
  if (!currentRowKey) {
    selectedMenuIds.value = selectedKeys;
    return;
  }

  isPermUpdating.value = true;
  selectedMenuIds.value = selectedKeys;

  const newChecked = new Set(checkedIds.value);
  const row = findPermRow(currentRowKey);

  if (actionType === 'check') {
    newChecked.add(currentRowKey);
    if (row) {
      // 勾选父菜单时，自动勾选其下所有按钮
      for (const btn of row.buttons) {
        newChecked.add(btn.id);
      }
      if (permCheckStrictly.value) {
        // 父子联动：同步勾选所有后代菜单 + 按钮
        for (const id of collectDescendantIds(row, true)) {
          newChecked.add(id);
        }
        for (const menuId of collectDescendantIds(row, false)) {
          if (!selectedMenuIds.value.includes(menuId)) {
            selectedMenuIds.value = [...selectedMenuIds.value, menuId];
          }
        }
      }
    }
  } else {
    newChecked.delete(currentRowKey);
    if (row) {
      for (const btn of row.buttons) {
        newChecked.delete(btn.id);
      }
      if (permCheckStrictly.value) {
        for (const id of collectDescendantIds(row, true)) {
          newChecked.delete(id);
        }
        const descendantMenuIds = collectDescendantIds(row, false);
        selectedMenuIds.value = selectedMenuIds.value.filter(
          (menuId) => !descendantMenuIds.includes(String(menuId)),
        );
      }
    }
  }

  checkedIds.value = [...newChecked];
  updateSelectAllState();
  isPermUpdating.value = false;
}

/// 按钮勾选回调：勾选按钮时自动勾选父菜单及其祖先菜单
function onPermButtonsChange(row: PermRow, newVal: Array<string | number | boolean>): void {
  if (isPermUpdating.value) return;

  const newValSet = new Set(newVal.map((val) => String(val)));
  const buttonIdSet = row.buttons.map((btn) => btn.id);

  for (const id of buttonIdSet) {
    const picked = checkedIds.value.includes(id);
    if (newValSet.has(id) && !picked) {
      checkedIds.value = [...checkedIds.value, id];
    } else if (!newValSet.has(id) && picked) {
      checkedIds.value = checkedIds.value.filter((checked) => checked !== id);
    }
  }

  const hasAnyButtonChecked = row.buttons.some((btn) => checkedIds.value.includes(btn.id));
  if (hasAnyButtonChecked && !selectedMenuIds.value.includes(row.id)) {
    selectedMenuIds.value = [...selectedMenuIds.value, row.id];
    if (permCheckStrictly.value) {
      // 父子联动：同时勾选所有祖先菜单
      const ancestors = collectAncestorIds(row.id);
      if (ancestors) {
        for (const ancestorId of ancestors) {
          if (!selectedMenuIds.value.includes(ancestorId)) {
            selectedMenuIds.value = [...selectedMenuIds.value, ancestorId];
          }
        }
      }
    }
  }

  updateSelectAllState();
}

/// 展开/折叠全部节点
function togglePermExpand(): void {
  permExpandAll.value = !permExpandAll.value;
  if (permExpandAll.value) {
    permTableRef.value?.expandAll();
  } else {
    permTableRef.value?.foldAll();
  }
}

/// 全选/取消全选
function togglePermSelectAll(): void {
  isPermUpdating.value = true;
  if (permSelectAll.value) {
    checkedIds.value = [];
    selectedMenuIds.value = [];
    permSelectAll.value = false;
  } else {
    checkedIds.value = collectAllPermIds(permTreeData.value);
    selectedMenuIds.value = collectAllMenuIds(permTreeData.value);
    permSelectAll.value = true;
  }
  isPermUpdating.value = false;
}

/// 树节点展开状态变化回调
function onPermExpandedChange(nodes: Array<string | number>): void {
  permExpandedKeys.value = nodes;
  if (nodes.length === 0) {
    permExpandAll.value = false;
  }
}

/// 保存权限（含菜单 + 按钮 ID）
async function handleSavePerms(): Promise<void> {
  if (!selectedRoleId.value) return;
  savePermLoading.value = true;
  try {
    await updateRoleMenus(selectedRoleId.value, checkedIds.value);
    void MessagePlugin.success('权限保存成功');
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '权限保存失败');
  } finally {
    savePermLoading.value = false;
  }
}

/// 打开新增弹窗
function openCreate(): void {
  formMode.value = 'create';
  editingId.value = '';
  Object.assign(formData, {
    role_code: '',
    role_name: '',
    description: '',
    sort_order: 0,
    status: 1,
  });
  formVisible.value = true;
}

/// 打开编辑弹窗，优先拉取角色详情
async function openEdit(row: RoleRes): Promise<void> {
  formMode.value = 'edit';
  editingId.value = row.id;
  try {
    const detail = await getRole(row.id);
    Object.assign(formData, {
      role_code: detail.role_code,
      role_name: detail.role_name,
      description: detail.description,
      sort_order: detail.sort_order,
      status: detail.status,
    });
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '角色详情加载失败');
    // 详情加载失败时回退到列表行数据，保证弹窗仍可编辑
    Object.assign(formData, {
      role_code: row.role_code,
      role_name: row.role_name,
      description: row.description,
      sort_order: row.sort_order,
      status: row.status,
    });
  }
  formVisible.value = true;
}

/// 提交新增/编辑表单
async function handleFormSubmit(): Promise<void> {
  const valid = await formRef.value?.validate();
  if (valid !== true) return;

  submitting.value = true;
  const payload: RoleSaveParams = {
    role_code: formData.role_code,
    role_name: formData.role_name,
    description: formData.description || undefined,
    sort_order: formData.sort_order,
    status: formData.status,
  };
  try {
    if (formMode.value === 'create') {
      await createRole(payload);
      void MessagePlugin.success('新增成功');
    } else {
      await updateRole(editingId.value, payload);
      void MessagePlugin.success('更新成功');
    }
    formVisible.value = false;
    await loadList();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    submitting.value = false;
  }
}

/// 删除角色；若删除的是当前选中角色则清空右侧详情
async function handleDelete(row: RoleRes): Promise<void> {
  try {
    await deleteRole(row.id);
    void MessagePlugin.success('删除成功');
    if (selectedRoleId.value === row.id) {
      selectedRoleId.value = '';
      selectedRole.value = null;
      permTreeData.value = [];
      selectedMenuIds.value = [];
      checkedIds.value = [];
      roleUsers.value = [];
      usersPagination.total = 0;
    }
    await loadList();
  } catch (error) {
    showError(error instanceof ApiError ? error.message : '删除失败');
  }
}

onMounted(() => {
  void loadList();
});
</script>

<style lang="less" scoped>
.role-page {
  .role-layout {
    display: flex;
    gap: 16px;
    // 视口高度减去布局头部（56px）与内容区上下内边距（32px）
    height: calc(100vh - 88px);
  }

  .role-left {
    width: 280px;
    min-width: 280px;
    border-right: 1px solid var(--td-component-stroke);
    overflow: hidden;
  }

  .role-list-card {
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

  .role-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--td-component-stroke);

    .role-title {
      font-size: 15px;
      font-weight: 600;
    }
  }

  .role-list-body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .role-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    cursor: pointer;
    border-left: 3px solid transparent;
    transition: all 0.2s;
    margin-bottom: 2px;

    &:hover {
      background: var(--td-bg-color-container-hover);
    }

    &.active {
      background: var(--td-brand-color-light);
      border-left-color: var(--td-brand-color);
    }
  }

  .role-item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;

    .role-item-name {
      font-size: 14px;
      font-weight: 500;
      color: var(--td-text-color-primary);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .role-item-code {
      font-size: 12px;
      color: var(--td-text-color-secondary);
    }
  }

  .role-item-actions {
    // 行内编辑/删除按钮始终可见，不依赖 hover
    display: flex;
    gap: 4px;
  }

  .role-right {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 16px;
    min-width: 0;
  }

  /// 右侧 t-card 撑满 role-right 高度，让 t-tabs 内容可滚动
  .role-right-card {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;

    :deep(.t-card__body) {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }

    :deep(.t-tabs) {
      flex: 1;
      display: flex;
      flex-direction: column;
      min-height: 0;
    }

    :deep(.t-tabs__content) {
      flex: 1;
      overflow-y: auto;
      padding-top: 12px;
    }

    :deep(.t-tab-panel) {
      display: flex;
      flex-direction: column;
      min-height: 0;
    }
  }

  .perm-tree-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .perm-controls {
    flex-shrink: 0;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--td-component-stroke);
    display: flex;
    justify-content: flex-start;
  }

  .perm-footer {
    flex-shrink: 0;
    padding: 12px 0;
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--td-component-stroke);
    margin-top: 0;
  }

  .perm-empty,
  .perm-placeholder {
    padding: 48px;
    text-align: center;
    color: var(--td-text-color-placeholder);
    font-size: 14px;
  }

  /// 让 t-loading 和 t-enhanced-table 在 perm-tree-wrapper 内撑满高度
  .perm-tree-wrapper :deep(.t-loading) {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .perm-tree-wrapper :deep(.t-enhanced-table) {
    flex: 1;
    min-height: 0;
  }

  .role-empty {
    padding: 32px;
    text-align: center;
    color: var(--td-text-color-placeholder);
  }

  .perm-button-group {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;

    :deep(.t-checkbox) {
      margin-right: 0;
      white-space: nowrap;
    }
  }

  /// 角色用户列表暂未接入接口，居中展示空态
  .users-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px 0;
  }
}
</style>
