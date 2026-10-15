import { MessagePlugin } from 'tdesign-vue-next';
import { ApiError } from '@/api/request';

/**
 * 全局错误兜底。
 *
 * 页面级请求错误已由各页面 `showError` 显式提示，这里只兜住**逃逸到全局**的异常，
 * 避免出现「操作失败却毫无反馈」的静默失败：
 *
 * - `unhandledrejection`：未被任何 `catch` 处理的 Promise 失败（含漏 catch 的请求）；
 * - `error`：未捕获的同步运行时异常，只记录不弹窗，避免把代码缺陷伪装成业务提示。
 *
 * 因页面已捕获的请求不会进入此监听，故不会与页面级提示重复弹窗。
 */
export function setupGlobalErrorHandler(): void {
  window.addEventListener('unhandledrejection', (event) => {
    const reason: unknown = event.reason;
    if (reason instanceof ApiError) {
      void MessagePlugin.error(reason.message);
      return;
    }
    // 非业务异常仅记录，交由开发排查
    console.error('[global] 未处理的 Promise 异常', reason);
  });

  window.addEventListener('error', (event) => {
    console.error('[global] 未捕获的运行时异常', event.error ?? event.message);
  });
}
