/// <reference types="vite/client" />

/** Vite 注入的环境变量（仅 `VITE_` 前缀的变量会被暴露到客户端） */
interface ImportMetaEnv {
  /** 页面标题 */
  readonly VITE_APP_TITLE: string;
  /** 接口基础路径，默认空串（接口路径在 api 模块中写全） */
  readonly VITE_APP_BASE_API: string;
  /** 部署子路径，默认根路径 */
  readonly VITE_APP_CONTEXT_PATH: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module '*.vue' {
  import type { DefineComponent } from 'vue';

  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}
