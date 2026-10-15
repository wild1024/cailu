import { resolve } from 'node:path';
import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';
import vueJsx from '@vitejs/plugin-vue-jsx';
import AutoImport from 'unplugin-auto-import/vite';
import Components from 'unplugin-vue-components/vite';
import { TDesignResolver } from 'unplugin-vue-components/resolvers';
import compression from 'vite-plugin-compression';

/** 后端服务地址：开发期通过 Vite 代理转发，规避浏览器跨域 */
const BACKEND_ORIGIN = 'http://127.0.0.1:8023';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd());

  return {
    // 部署在子路径时由 VITE_APP_CONTEXT_PATH 指定，默认根路径
    base: env.VITE_APP_CONTEXT_PATH || '/',
    plugins: [
      vue(),
      vueJsx(),
      // 自动引入 vue / vue-router / pinia 的 API（ref、computed 等），减少样板代码
      AutoImport({
        imports: ['vue', 'vue-router', 'pinia'],
        dts: 'src/types/auto-imports.d.ts',
      }),
      // 按需引入 TDesign 组件（模板中 t-xxx 自动解析并注册）
      Components({
        resolvers: [TDesignResolver({ library: 'vue-next' })],
        dts: 'src/types/components.d.ts',
      }),
      compression({ algorithm: 'gzip', ext: '.gz', threshold: 10240 }),
    ],
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    css: {
      preprocessorOptions: {
        less: {
          javascriptEnabled: true,
        },
      },
    },
    server: {
      // 与后端 config.toml 的 [middleware.cors] allow_origins 保持一致
      port: 5173,
      host: '0.0.0.0',
      proxy: {
        // 业务接口与健康探针统一转发到后端
        '/api': { target: BACKEND_ORIGIN, changeOrigin: true },
        '/health': { target: BACKEND_ORIGIN, changeOrigin: true },
      },
    },
    build: {
      outDir: 'dist',
      sourcemap: false,
      chunkSizeWarningLimit: 1500,
      rollupOptions: {
        output: {
          // 仅把体积大、版本稳定的库单独拆包；不再兜底所有 node_modules，
          // 否则 TDesign 与其依赖会被切到两个互相引用的 chunk（rollup 报 Circular chunk）
          manualChunks(id: string) {
            if (id.includes('node_modules/echarts') || id.includes('node_modules/zrender')) {
              return 'echarts';
            }
            if (id.includes('node_modules/tdesign')) return 'tdesign';
            return undefined;
          },
        },
      },
    },
  };
});
