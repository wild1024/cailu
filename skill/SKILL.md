---
name: cailu
description: >-
  菜芦通用后台系统 基础版（Cailu）项目级 AI 知识包：Rust+alun 后端 / Vue3+TDesign 前端。
  当需要在 cailu 下编写/审查/调试后端 Rust、前端 Vue、迁移 SQL、新增业务模块、鉴权/JWT 时加载，优先于逐文件检索，节省 token。
agent_created: true
---

# 菜芦通用后台系统 基础版 — AI 速查（项目级）

> 配套 `AGENTS.md`（操作指令）与 `llms.txt`（文件地图）。本文件为浓缩知识包，加载后即用。


## 高频命令
```bash
cd backend && cargo run            # 自动迁移+启动 :8023
cd backend && cargo check
cd frontend && pnpm install && pnpm dev   # :5173
bash scripts/dev.sh                # 依赖容器+后端+前端
bash scripts/check.sh              # 提交前 fmt/clippy/typecheck
```
默认账号 `admin/Admin@123456`；接口前缀 `/api`；健康检查 `/health`（免鉴权）。

## 后端核心约定
- **四件套**：每模块 `mod.rs+handler.rs+dto.rs+model.rs`；业务编排 handler、数据访问 model，**不设 service 层**。
- **二开容器 `business/`**：后端 `src/business/<模块>/`、前端 `api/business/`、`views/business/<模块>/`；与 `system/`、`support/` 平级，不侵入底座；菜单种子 `module_code='business'`。
- **依赖铁律**：`Cargo.toml` 只声明 `alun`（必要时 `alun-utils`）；**不写** `alun-db/alun-db-core/alun-web/alun-config`。业务代码 `use alun::prelude::*`。
- **feature**：默认集 `core`（非 full）。用 DB 须显式 `postgres/mysql/sqlite`。预设 `mini⊂lite⊂full`，常用 `lite`（core+postgres+cache+plugin+fs-local）。点单：`fs-local`/`fs-minio`、`xlsx`、`sse`、`metrics`、`template`、`xss`。
- **错误出口三名字**：handler `ResResult<T>`、model `DbResult<T>`、启动/插件/任务 `alun::Result<T>`；handler 内 `DbError/io::Error/alun_core::Error` 直接 `?`，勿写多余 `.map_err(ApiError::internal(...))`。
- **迁移号段**（动手前 `ls backend/migrations/*/` 核对）：基础版 `V0017` 封顶；4 位补零，已应用迁移指纹校验不可改；种子 `ON CONFLICT` 幂等。
- **路由宏**：`#[alun::get/post(...)]` + `#[alun::permission(path,method,permission)]`；路径不含 `/api`；`/health` 手动注册。
- **事务 RAII**：`db().transaction(|tx| async move { ... }).await`，`Err` 自动回滚。

## 前端核心约定
- 目录 `api/{system,support,business}` + `views/{...}` + `router/index.ts`（静态路由 + `meta.permission`）+ `stores/`（user/dict）。
- **API 路径写全 `/api/...`**（漏前缀命中 SPA fallback 返回 HTML → "响应体格式不符合"）。
- **TDesign Grid 12 列制**：`t-col-xl-2`=6 个/行，`-xl-3`=4 个/行；`t-col-xs-24` 不存在（最大 `xs-12`）。
- 图标：`user-check`→`user-checked`、`speed`→`rocket`。
- 列表页范本：`views/system/user/index.vue`；请求封装：`api/request.ts`（401 自动刷新）。

## 高频踩坑
1. 接口/报表不显示：前端 api 漏 `/api`。
2. TDesign 图标：`user-check`→`user-checked`、`speed`→`rocket`。

## 关键文件指针
- 启动链：`backend/src/lib.rs`
- 用户模块范本：`backend/src/system/user/{handler,dto,model}.rs`
- 菜单种子：`backend/migrations/seed/S__menu_data.sql`
- 配置：`backend/config/config.toml`
- 工程脚本：`scripts/{dev,check,build,deploy,init-db}.sh`
- 二开文档集：`docs/README.md`
- 框架源码（同级）：`../../../alun/`
