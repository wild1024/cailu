<template>
  <t-dialog
    :visible="visible"
    header="联系我们"
    :footer="false"
    width="380px"
    destroy-on-close
    @update:visible="(next: boolean) => emit('update:visible', next)"
  >
    <div class="contact-dialog">
      <p class="contact-dialog__text">{{ text }}</p>
      <figure class="contact-dialog__qr">
        <img src="/wechat-qrcode.jpg" alt="弘纹公众号二维码" />
        <figcaption>扫码关注「弘纹」公众号</figcaption>
      </figure>
      <p class="contact-dialog__hint">获取使用技巧、版本更新提醒与技术支持。</p>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
/**
 * 「联系我们」弹窗：统一承载公众号二维码。
 *
 * 二维码是 `public/wechat-qrcode.jpg`（二开直接替换该文件即可，不用改代码）。
 */
withDefaults(
  defineProps<{
    /** 是否显示。 */
    visible: boolean;
    /** 渠道说明文案。 */
    text?: string;
  }>(),
  {
    text: '技术支持、商务合作与定制开发，请关注公众号联系作者，或在项目仓库提交 Issue。',
  },
);

const emit = defineEmits<{
  (event: 'update:visible', value: boolean): void;
}>();
</script>

<style scoped lang="less">
.contact-dialog {
  text-align: center;

  &__text {
    margin: 0 0 20px;
    font-size: 14px;
    line-height: 1.7;
    color: var(--td-text-color-secondary);
    text-align: left;
  }

  &__qr {
    margin: 0;
    padding: 12px;
    border: 1px solid var(--td-component-stroke);
    border-radius: 12px;
    background: #fff;

    img {
      display: block;
      width: 200px;
      height: 200px;
      margin: 0 auto;
      // 二维码需要留白（quiet zone），缩放时不能裁切
      object-fit: contain;
    }

    figcaption {
      margin-top: 10px;
      font-size: 13px;
      color: var(--td-text-color-primary);
    }
  }

  &__hint {
    margin: 12px 0 0;
    font-size: 12px;
    color: var(--td-text-color-placeholder);
  }
}
</style>
