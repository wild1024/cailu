<template>
  <div class="login">
    <div class="login__panel">
      <!-- 左侧品牌区：柔和渐变底 + 一句话定位 -->
      <section class="login__brand">
        <div class="login__brand-glow" aria-hidden="true"></div>

        <div class="login__brand-inner">
          <!-- 主内容贴顶：品牌识别（logo + 名称）居上，页脚保持沉底 -->
          <div class="login__brand-main">
            <div class="login__brand-head">
              <span class="login__badge">
                <img src="/logo.png" alt="菜芦" />
              </span>
              <p class="login__brand-name">菜芦管理</p>
            </div>

            <h2 class="login__brand-title">AI Coding 的管理底座</h2>
            <p class="login__brand-desc">极小化 Token 消耗 · 极大化 Attention 浓度<br>稳定生成高品质代码</p>
          </div>

          <p class="login__brand-footer">Rust + alun · Vue 3 + TDesign</p>
        </div>
      </section>

      <!-- 右侧表单区 -->
      <section class="login__form-side">
        <!-- 右侧 Logo 仅小屏出现：大屏由左侧品牌区承载，两边同时出现会重复 -->
        <span class="login__badge login__badge--form">
          <img src="/logo.png" alt="菜芦" />
        </span>

        <header class="login__form-head">
          <h1 class="login__form-title">账号登录</h1>
          <p class="login__form-subtitle">请使用后台分配的用户名与密码登录</p>
        </header>

        <t-alert v-if="errorMessage" class="login__alert" theme="error" :message="errorMessage" />

        <t-form :data="formData" :rules="formRules" label-width="0" @submit="handleSubmit">
          <t-form-item name="username">
            <t-input
              v-model="formData.username"
              size="large"
              clearable
              placeholder="请输入用户名"
              autocomplete="username"
            >
              <template #prefix-icon><t-icon name="user" /></template>
            </t-input>
          </t-form-item>
          <t-form-item name="password">
            <t-input
              v-model="formData.password"
              type="password"
              size="large"
              clearable
              placeholder="请输入密码"
              autocomplete="current-password"
            >
              <template #prefix-icon><t-icon name="lock-on" /></template>
            </t-input>
          </t-form-item>
          <t-form-item>
            <t-button block theme="primary" size="large" type="submit" :loading="submitting">
              登 录
            </t-button>
          </t-form-item>
        </t-form>

        <p class="login__hint">默认账号：admin / Admin@123456</p>
        <p class="login__signup">还没有账号？<b>联系管理员开通</b></p>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import type { FormRules } from 'tdesign-vue-next';
import type { LoginParams } from '@/api/system/auth-api';
import { ApiError } from '@/api/request';
import { useUserStore } from '@/stores/user';

const route = useRoute();
const router = useRouter();
const userStore = useUserStore();

/** 登录表单数据 */
const formData = reactive<LoginParams>({ username: '', password: '' });
/** 登录表单校验规则 */
const formRules: FormRules<LoginParams> = {
  username: [{ required: true, message: '请输入用户名' }],
  password: [{ required: true, message: '请输入密码' }],
};
/** 提交中标记 */
const submitting = ref(false);
/** 登录失败提示 */
const errorMessage = ref('');

/** 提交登录：失败提示留在表单内，成功后按 redirect 参数回跳 */
async function handleSubmit(): Promise<void> {
  // 账号密码去掉首尾空白（复制粘贴常带上不可见空格），后端登录接口同样会 trim 兜底
  const username = formData.username.trim();
  const password = formData.password.trim();
  submitting.value = true;
  errorMessage.value = '';
  try {
    await userStore.login({ username, password });
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/dashboard';
    await router.replace(redirect);
  } catch (error) {
    errorMessage.value = error instanceof ApiError ? error.message : '登录失败，请稍后重试';
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped lang="less">
.login {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  // 用 vh 而不是 100%：<router-view> 的包裹层不保证撑满，百分比会塌成内容高度
  min-height: 100vh;
  padding: 32px 24px;
  // 整体柔和渐变：淡蓝 → 淡紫 → 淡青，低饱和，长时间看不刺眼
  background: linear-gradient(135deg, #eff4ff 0%, #f6f2ff 50%, #e9f6fb 100%);
}

.login__panel {
  display: flex;
  width: 100%;
  max-width: 960px;
  // 给卡片一个最小高度，左右两栏才有共同的体量，右侧内容垂直居中才有意义
  min-height: 520px;
  overflow: hidden;
  background-color: var(--td-bg-color-container);
  border: 1px solid rgb(255 255 255 / 70%);
  border-radius: 16px;
  box-shadow: 0 18px 48px rgb(94 112 180 / 14%);
}

// ---------- 左侧品牌区 ----------
.login__brand {
  position: relative;
  flex: 1 1 auto;
  min-width: 0;
  padding: 48px 44px;
  overflow: hidden;
  // 柔和渐变底，与页面外框同一色系但略深一档，形成层次
  background: linear-gradient(155deg, #e6edff 0%, #efeaff 55%, #e5f4fa 100%);
}

// 柔光装饰：不抢内容，只负责让渐变不死板
.login__brand-glow {
  position: absolute;
  top: -120px;
  right: -140px;
  width: 320px;
  height: 320px;
  background: radial-gradient(circle, rgb(122 142 255 / 22%) 0%, rgb(122 142 255 / 0%) 70%);
  pointer-events: none;
}

.login__brand-inner {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
}

// 主内容组贴顶展示：品牌识别（logo + 名称）在视觉重心上方，标题与描述紧随其后
.login__brand-main {
  margin: 0;
}

.login__brand-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

// 方形圆角徽章：圆角矩形裁切 + 外阴影，与整体卡片风格呼应
.login__badge {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  overflow: hidden;
  background-color: rgb(255 255 255 / 92%);
  border-radius: 12px;
  box-shadow:
    inset 0 0 0 1px rgb(255 255 255 / 80%),
    0 6px 16px rgb(94 112 180 / 22%);

  > img {
    width: 70%;
    height: 70%;
    object-fit: contain;
  }
}

// 右侧表单区 Logo：与左侧品牌区徽章**分开控制尺寸**——左侧 40px，这里 52px
// （右侧是登录后进入系统的唯一品牌标识，需要更醒目；同为方形圆角 + 阴影）
.login__badge--form {
  display: none;
  width: 68px;
  height: 68px;
  border-radius: 14px;

  > img {
    width: 80%;
    height: 80%;
  }
}

.login__brand-name {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  line-height: 1.3;
  letter-spacing: 1px;
  color: var(--td-text-color-primary);
}

.login__brand-title {
  margin: 128px 0 12px;
  font-size: 24px;
  font-weight: 600;
  line-height: 1.4;
  color: var(--td-text-color-primary);
}

.login__brand-desc {
  margin: 0;
  font-size: 14px;
  line-height: 1.7;
  color: var(--td-text-color-secondary);
}

// margin-top auto：主内容贴顶后，页脚由弹性布局压到品牌区底部
.login__brand-footer {
  margin-top: auto;
  padding-top: 24px;
  font-size: 12px;
  color: var(--td-text-color-placeholder);
}

// ---------- 右侧表单区 ----------
.login__form-side {
  display: flex;
  flex-direction: column;
  justify-content: center;
  flex: 0 0 400px;
  padding: 48px 40px;
  background-color: var(--td-bg-color-container);
}

.login__form-head {

  margin-bottom: 28px;
}

.login__form-title {
  margin: 0 0 6px;
  font-size: 22px;
  font-weight: 600;
  color: var(--td-text-color-primary);
}

.login__form-subtitle {
  margin: 0;
  font-size: 13px;
  color: var(--td-text-color-placeholder);
}

.login__alert {
  margin-bottom: 16px;
}

.login__hint {
  margin: 20px 0 0;
  font-size: 12px;
  text-align: center;
  color: var(--td-text-color-placeholder);
}

.login__signup {
  margin: 10px 0 0;
  font-size: 12px;
  text-align: center;
  color: var(--td-text-color-placeholder);

  b {
    font-weight: 500;
    color: var(--td-brand-color);
  }
}

// ---------- 响应式 ----------
// 中等屏：品牌区收紧留白，避免文字被挤到换行
@media (max-width: 1024px) {
  .login__panel {
    max-width: 840px;
    min-height: 480px;
  }

  .login__form-side {
    flex-basis: 360px;
    padding: 40px 32px;
  }

  .login__brand {
    padding: 40px 32px;

    .login__brand-title {
      margin-top: 128px;
      font-size: 21px;
    }
  }
}

// 小屏：左侧品牌区整体隐藏，整页只保留表单，不做上下堆叠
@media (max-width: 768px) {
  .login {
    padding: 0;
  }

  .login__panel {
    display: block;
    // 左侧隐藏后卡片不再需要最小高度，改由表单区占满一屏
    min-height: 0;
    max-width: none;
    border: 0;
    border-radius: 0;
    box-shadow: none;
  }

  .login__brand {
    display: none;
  }

  .login__form-side {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 100vh;
    padding: 32px 24px;
  }

  // 左侧品牌区隐藏后，右侧补同款方形圆角徽章承载品牌识别
  .login__badge--form {
    display: inline-flex;
    margin: 0 auto 22px;

  }
}
</style>
