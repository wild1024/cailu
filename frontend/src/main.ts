import { createApp } from 'vue';
import { createPinia } from 'pinia';
import 'tdesign-vue-next/es/style/index.css';

import App from '@/App.vue';
import router from '@/router';
import { setupDirectives } from '@/directives';
import { setupGlobalErrorHandler } from '@/utils/global-error';
import '@/style/index.less';

// 注册全局错误兜底：捕获未被页面处理的异常，避免静默失败
setupGlobalErrorHandler();

const app = createApp(App);

// 指令内部依赖 Pinia 的 store，需在 use(createPinia()) 之后再创建使用
app.use(createPinia());
setupDirectives(app);
app.use(router);
app.mount('#app');
