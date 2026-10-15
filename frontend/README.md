# 菜芦通用后台系统 · 前端（frontend）

基于 **Vue 3 + TypeScript + Vite + TDesign** 的通用后台管理前端，与后端工程 [`cailu`](../backend) 配套使用。
覆盖登录鉴权、用户 / 角色 / 菜单 / 部门、字典 / 参数 / 文件 / 日志 / 定时任务 / 在线用户等管理页面。

> 后端接口统一前缀 `/api`，健康探针 `/health`；开发期由 Vite 代理转发，生产期由 nginx 反向代理。

---

## 一、环境准备

| 依赖 | 版本要求 | 说明 |
|------|----------|------|
| Node.js | **≥ 22.13.0** | 低版本下 corepack 的 pnpm shim 会崩溃 |
| pnpm | **10.5.0** | 已由 `package.json` 的 `packageManager` 固定，无需手动指定 |

```bash
# 如本机使用 nvm，先切到满足要求的 Node
export PATH="$HOME/.nvm/versions/node/v24.14.0/bin:$PATH"
node -v      # 应 ≥ 22.13.0
```

---

## 二、快速开始

```bash
# 1. 安装依赖（首次会自动执行 esbuild 构建脚本）
pnpm install

# 2. 启动开发服务器（默认 http://localhost:5173）
pnpm dev
```

启动前请确认后端已在 `http://127.0.0.1:8023` 运行（`cargo run`）。
浏览器打开 `http://localhost:5173`，用后端种子账号登录即可。

### 常用命令

| 命令 | 作用 |
|------|------|
| `pnpm dev` | 启动开发服务器（热更新） |
| `pnpm typecheck` | TypeScript 类型检查（提交前必跑） |
| `pnpm build` | 类型检查 + 生产构建，产物输出到 `dist/` |
| `pnpm preview` | 本地预览构建产物 |

---

## 三、目录结构

```text
frontend/
├── deploy/nginx.conf       # 生产 nginx 站点配置
├── index.html
├── vite.config.ts          # 开发代理 /api、/health → 127.0.0.1:8023
├── .env.development        # 开发环境变量
├── .env.production         # 生产环境变量
└── src/
    ├── api/                # 接口封装（按后端模块划分，路径写全 /api/...）
    ├── router/             # 路由表与权限守卫
    ├── stores/             # Pinia：user（会话/权限/菜单）、dict（字典缓存）
    ├── layouts/            # 后台整体布局（侧边栏 / 顶栏 / 内容区）
    ├── views/              # 页面：system/*、support/*
    ├── components/         # 通用组件：部门树选择 / 字典标签 / 字典下拉 / 上传
    ├── utils/              # 格式化、菜单路径等纯函数
    └── types/              # 与后端 DTO 对齐的 TS 类型
```

---

## 四、接口与鉴权约定

- **统一响应体**：`{ "code": 0, "msg": "ok", "data": ... }`，`code !== 0` 视为业务错误；
  分页 `data` 形如 `{ list, total, page, page_size }`。
- **令牌**：请求头 `Authorization: Bearer <access_token>`；`access_token` 有效期 2 小时。
- **401 自动刷新**：拦截器用 `refresh_token` 换新令牌并自动重放原请求；
  并发 401 会合并为一次刷新；刷新失败则清空本地令牌并跳回登录页。
- **路由守卫**：未登录跳登录页；已登录按 `meta.permission` 权限码校验，无权限回落首页。

---

## 五、构建与部署

### 1. 构建产物

```bash
pnpm build           # 产出 dist/（含 .gz 预压缩文件）
```

### 2. nginx 托管

使用仓库内 [`deploy/nginx.conf`](deploy/nginx.conf)：

```bash
# 1. 上传产物到服务器
scp -r dist/* user@server:/var/www/cailu/frontend/dist/

# 2. 放置 nginx 配置并按需修改 root 路径与后端地址
sudo cp deploy/nginx.conf /etc/nginx/conf.d/cailu-frontend.conf
sudo nginx -t && sudo nginx -s reload
```

配置要点：

- `location /` 使用 `try_files $uri $uri/ /index.html` 兜底，解决刷新子路由 404；
- `/api/` 与 `/health` 反向代理到后端 `127.0.0.1:8023`，**保留 `/api` 前缀**；
- `index.html` 不缓存，`/assets/` 长缓存，发版即时生效。

### 3. 生产环境变量（后端）

生产敏感值不落配置文件，由环境变量注入（详见后端 `config/config-prod.toml`）：

```text
ALUN_MIDDLEWARE_AUTH_JWT_SECRET=<jwt 密钥，至少 32 字符>
ALUN_DATABASE_PASSWORD=<数据库密码>
ALUN_SERVER_LISTEN=0.0.0.0:8023
```

---

## 六、常见问题（FAQ）

**Q1：`pnpm install` 报错或 pnpm 无法启动？**
A：多为 Node 版本过低导致 corepack shim 崩溃。请升级到 Node ≥ 22.13.0，再执行 `corepack enable` 后重试。

**Q2：页面能打开，但接口全部 404 或跨域？**
A：确认后端已在 `127.0.0.1:8023` 启动。开发期接口走后端 `[middleware.cors].allow_origins`，
需包含 `http://localhost:5173`；生产期由 nginx 反代，不依赖 CORS。

**Q3：登录成功但侧边栏没有菜单？**
A：菜单由后端 `GET /api/system/auth/menus` 下发，仅有当前用户被授权的菜单会显示。
请确认该角色已在「角色管理」中勾选菜单授权。

**Q4：接口偶发返回 401，会自动登出吗？**
A：不会立即登出。拦截器先尝试用 `refresh_token` 刷新并重放原请求；
仅当刷新令牌也失效时才清空会话并跳登录页。

**Q5：想部署到子路径（如 `/admin/`）？**
A：修改 `.env.production` 的 `VITE_APP_CONTEXT_PATH=/admin/` 重新构建，
并让 nginx 的 `location` 与 `root` 与该子路径对齐。

**Q6：`pnpm build` 报类型错误怎么办？**
A：`build` 会先跑 `vue-tsc` 类型检查。可先单独执行 `pnpm typecheck` 定位文件，修复后再构建。
