<template>
  <div class="profile-page">
    <t-row :gutter="[24, 24]">
      <!-- 左列：问候 + 概览 + 功能 tabs -->
      <t-col :flex="3">
        <div class="profile-greeting">
          <div>
            Hi，{{ userStore.displayName || '用户' }}
            <span class="profile-greeting__regular">{{ greetingSuffix }}</span>
          </div>
        </div>

        <t-card class="profile-card" :bordered="false" title="个人信息">
          <t-descriptions :column="4" item-layout="vertical">
            <t-descriptions-item label="用户名">{{ userStore.userInfo?.username || '-' }}</t-descriptions-item>
            <t-descriptions-item label="姓名">{{ userStore.userInfo?.real_name || '-' }}</t-descriptions-item>
            <t-descriptions-item label="邮箱">{{ userStore.userInfo?.email || '-' }}</t-descriptions-item>
            <t-descriptions-item label="手机号">{{ userStore.userInfo?.mobile || '-' }}</t-descriptions-item>
          </t-descriptions>
        </t-card>

        <t-card class="profile-card" :bordered="false">
          <t-tabs v-model="activeTab" @change="onTabChange">
            <t-tab-panel value="basic" label="基本信息">
              <t-form
                ref="profileFormRef"
                :data="profileForm"
                :rules="profileRules"
                label-width="80px"
                class="profile-form"
                @submit="submitProfile"
              >
                <t-form-item label="用户名">
                  <t-input :value="userStore.userInfo?.username" disabled />
                </t-form-item>
                <t-form-item label="姓名" name="real_name">
                  <t-input v-model="profileForm.real_name" placeholder="请输入姓名" />
                </t-form-item>
                <t-form-item label="邮箱" name="email">
                  <t-input v-model="profileForm.email" placeholder="请输入邮箱" />
                </t-form-item>
                <t-form-item label="手机号" name="mobile">
                  <t-input v-model="profileForm.mobile" placeholder="请输入手机号" />
                </t-form-item>
                <t-form-item>
                  <t-button theme="primary" type="submit" :loading="profileSubmitting">保存</t-button>
                </t-form-item>
              </t-form>
            </t-tab-panel>

            <t-tab-panel value="security" label="安全设置">
              <t-form
                ref="passwordFormRef"
                :data="passwordForm"
                :rules="passwordRules"
                label-width="90px"
                class="profile-form"
                @submit="submitPassword"
              >
                <t-form-item label="原密码" name="old_password">
                  <t-input v-model="passwordForm.old_password" type="password" placeholder="请输入原密码" />
                </t-form-item>
                <t-form-item label="新密码" name="new_password">
                  <t-input v-model="passwordForm.new_password" type="password" placeholder="请输入新密码" />
                </t-form-item>
                <t-form-item label="确认密码" name="confirm_password">
                  <t-input v-model="passwordForm.confirm_password" type="password" placeholder="请再次输入新密码" />
                </t-form-item>
                <t-form-item>
                  <t-button theme="primary" type="submit" :loading="passwordSubmitting">修改密码</t-button>
                </t-form-item>
              </t-form>
            </t-tab-panel>

            <t-tab-panel value="logs" label="登录日志">
              <t-table
                row-key="id"
                :data="loginLogs"
                :columns="loginLogColumns"
                :loading="loginLogLoading"
                :pagination="loginLogPagination"
                :hover="true"
                :stripe="true"
                class="profile-form"
                @page-change="onLoginLogPageChange"
              >
                <template #auth_type="{ row }">{{ authTypeLabel(row.auth_type) }}</template>
                <template #auth_status="{ row }">
                  <t-tag :theme="row.auth_status === 'SUCCESS' ? 'success' : 'danger'" variant="light">
                    {{ row.auth_status === 'SUCCESS' ? '成功' : '失败' }}
                  </t-tag>
                </template>
                <template #created_at="{ row }">{{ formatDateTime(row.created_at) }}</template>
              </t-table>
            </t-tab-panel>
          </t-tabs>
        </t-card>
      </t-col>

      <!-- 右列：用户名片 -->
      <t-col :flex="1">
        <t-card class="profile-card profile-intro" :bordered="false">
          <t-avatar size="80px">
            <template #icon><t-icon name="user" /></template>
          </t-avatar>
          <div class="profile-intro__name">{{ userStore.displayName || '用户' }}</div>
          <div class="profile-intro__role">{{ roleText || '普通用户' }}</div>
        </t-card>
      </t-col>
    </t-row>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import type {
  FormInstanceFunctions,
  FormRules,
  PrimaryTableCol,
} from 'tdesign-vue-next';
import {
  changePassword,
  fetchMyLoginLogs,
  updateProfile,
  type AuthLogRow,
} from '@/api/system/auth-api';
import { ApiError } from '@/api/request';
import { useUserStore } from '@/stores/user';
import { formatDateTime } from '@/utils/format';

/** tabs 取值：基本信息 / 安全设置 / 登录日志 */
type ProfileTab = 'basic' | 'security' | 'logs';

/** 基本信息表单 */
interface ProfileForm {
  /** 真实姓名 */
  real_name: string;
  /** 邮箱 */
  email: string;
  /** 手机号 */
  mobile: string;
}

/** 修改密码表单 */
interface PasswordForm {
  /** 原密码 */
  old_password: string;
  /** 新密码 */
  new_password: string;
  /** 确认密码 */
  confirm_password: string;
}

const userStore = useUserStore();

/** 当前选中的 tab */
const activeTab = ref<ProfileTab>('basic');

/** 按时段生成问候语后缀 */
const greetingSuffix = computed<string>(() => {
  const hour = new Date().getHours();
  if (hour < 12) {
    return '上午好';
  }
  if (hour < 14) {
    return '中午好';
  }
  if (hour < 18) {
    return '下午好';
  }
  return '晚上好';
});

/** 角色文本（多个角色以「、」连接） */
const roleText = computed<string>(() => userStore.roles.join('、'));

// ==================== 基本信息 ====================

const profileFormRef = ref<FormInstanceFunctions | null>(null);
const profileSubmitting = ref(false);
const profileForm = reactive<ProfileForm>({ real_name: '', email: '', mobile: '' });

/** 基本信息校验：邮箱为可选项，填写时需符合格式 */
const profileRules: FormRules<ProfileForm> = {
  email: [{ email: true, message: '邮箱格式不正确' }],
};

/** 用当前用户信息回填表单 */
function fillProfileForm(): void {
  const info = userStore.userInfo;
  profileForm.real_name = info?.real_name ?? '';
  profileForm.email = info?.email ?? '';
  profileForm.mobile = info?.mobile ?? '';
}

async function submitProfile(): Promise<void> {
  const valid = await profileFormRef.value?.validate();
  if (valid !== true) {
    return;
  }
  profileSubmitting.value = true;
  try {
    await updateProfile({
      real_name: profileForm.real_name,
      email: profileForm.email,
      mobile: profileForm.mobile,
    });
    // 重新拉取用户信息，保证顶栏与概览同步刷新
    await userStore.loadProfile();
    fillProfileForm();
    void MessagePlugin.success('个人资料已保存');
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '保存失败');
  } finally {
    profileSubmitting.value = false;
  }
}

// ==================== 修改密码 ====================

const passwordFormRef = ref<FormInstanceFunctions | null>(null);
const passwordSubmitting = ref(false);
const passwordForm = reactive<PasswordForm>({
  old_password: '',
  new_password: '',
  confirm_password: '',
});

/** 修改密码校验：两次输入需一致，新密码不少于 6 位 */
const passwordRules: FormRules<PasswordForm> = {
  old_password: [{ required: true, message: '请输入原密码' }],
  new_password: [
    { required: true, message: '请输入新密码' },
    { min: 6, message: '新密码长度不少于 6 位' },
  ],
  confirm_password: [
    { required: true, message: '请再次输入新密码' },
    {
      validator: (value: string) => value === passwordForm.new_password,
      message: '两次输入的密码不一致',
    },
  ],
};

async function submitPassword(): Promise<void> {
  const valid = await passwordFormRef.value?.validate();
  if (valid !== true) {
    return;
  }
  passwordSubmitting.value = true;
  try {
    await changePassword({
      old_password: passwordForm.old_password,
      new_password: passwordForm.new_password,
    });
    passwordForm.old_password = '';
    passwordForm.new_password = '';
    passwordForm.confirm_password = '';
    void MessagePlugin.success('密码已修改');
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '密码修改失败');
  } finally {
    passwordSubmitting.value = false;
  }
}

// ==================== 登录日志 ====================

const loginLogs = ref<AuthLogRow[]>([]);
const loginLogLoading = ref(false);
const loginLogPagination = reactive({ current: 1, pageSize: 10, total: 0 });
const loginLogColumns: PrimaryTableCol[] = [
  { colKey: 'ip', title: 'IP', width: 140 },
  { colKey: 'auth_type', title: '登录类型', width: 110, cell: 'auth_type' },
  { colKey: 'auth_status', title: '状态', width: 90, cell: 'auth_status' },
  { colKey: 'error_message', title: '错误信息', ellipsis: true },
  { colKey: 'login_duration', title: '耗时(ms)', width: 100 },
  { colKey: 'created_at', title: '时间', width: 180, cell: 'created_at' },
];

/** 登录类型展示文案 */
function authTypeLabel(type?: string | null): string {
  const map: Record<string, string> = {
    LOGIN: '登录',
    LOGOUT: '登出',
    REFRESH_TOKEN: '刷新令牌',
  };
  return (type && map[type]) || type || '-';
}

async function loadLoginLogs(): Promise<void> {
  loginLogLoading.value = true;
  try {
    const data = await fetchMyLoginLogs({
      page: loginLogPagination.current,
      page_size: loginLogPagination.pageSize,
    });
    loginLogs.value = data.list;
    loginLogPagination.total = data.total;
  } catch (error) {
    void MessagePlugin.error(error instanceof ApiError ? error.message : '登录日志加载失败');
  } finally {
    loginLogLoading.value = false;
  }
}

function onLoginLogPageChange(pageInfo: { current: number; pageSize: number }): void {
  loginLogPagination.current = pageInfo.current;
  loginLogPagination.pageSize = pageInfo.pageSize;
  void loadLoginLogs();
}

/** 切换到登录日志 tab 时懒加载，避免无谓请求 */
function onTabChange(value: string | number): void {
  if (value === 'logs' && loginLogs.value.length === 0) {
    void loadLoginLogs();
  }
}

onMounted(() => {
  fillProfileForm();
});
</script>

<style scoped lang="less">
.profile-page {
  padding: 16px;
}

.profile-greeting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  margin-bottom: 16px;
  font-size: 20px;
  font-weight: 600;
  color: var(--td-text-color-primary);
  background-color: var(--td-bg-color-container);
  border-radius: 10px;

  &__regular {
    margin-left: 6px;
    font-size: 14px;
    font-weight: 400;
    color: var(--td-text-color-secondary);
  }
}

.profile-card {
  margin-bottom: 16px;

  &:last-child {
    margin-bottom: 0;
  }
}

.profile-form {
  margin-top: 16px;
}

.profile-intro {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;

  &__name {
    margin-top: 16px;
    font-size: 16px;
    font-weight: 600;
    color: var(--td-text-color-primary);
  }

  &__role {
    margin-top: 6px;
    font-size: 13px;
    color: var(--td-text-color-secondary);
  }
}
</style>
