<template>
  <div class="result-page">
    <div class="result-page__content">
      <t-icon name="lock-on" size="80px" class="result-page__icon" />
      <h2 class="result-page__code">403</h2>
      <p class="result-page__desc">抱歉，您没有权限访问该页面</p>
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
