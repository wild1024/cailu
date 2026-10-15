<template>
  <div class="result-page">
    <div class="result-page__content">
      <t-icon name="error-circle" size="80px" class="result-page__icon result-page__icon--error" />
      <h2 class="result-page__code">500</h2>
      <p class="result-page__desc">抱歉，服务器出现异常，请稍后重试</p>
      <t-button theme="primary" @click="goHome">返回首页</t-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';

const router = useRouter();

/** 返回首页 */
function goHome(): void {
  void router.push('/dashboard');
}
</script>

<style scoped lang="less">
@import '../result.less';
</style>
