import type { App } from 'vue';
import { privilege } from '@/directives/permission/privilege';

/// 注册全局自定义指令。
///
/// # 参数
/// - `app`：Vue 应用实例，用于挂载 `v-privilege` 等指令。
export function setupDirectives(app: App): void {
  app.directive('privilege', privilege);
}
