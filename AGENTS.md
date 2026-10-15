# AGENTS.md — 菜芦通用后台系统（基础版 Cailu）

给 AI 编码 Agent 的项目说明。读完本文件即可安全动手；深入约定见 `skill/SKILL.md`、`docs/README.md`（二开文档集入口）与各分册。

## 项目是什么
基于 **Rust + Alun 框架（alun）+ Vue 3 + TDesign** 的通用后台底座（基础版）。提供用户 / 角色 / 菜单 / 字典 / 配置 / 文件 / 日志 / 在线用户 / 定时任务等通用后台能力。

> 专业版 `cailu-pro`（数据权限 / 工作流 / 对象存储 / 导入导出）与企业版 `cailu-saas`（多租户）是**独立仓库**，不在本目录。

## 技术栈
- 后端：Rust 1.94.1+ / edition 2024，框架 `alun`（门面 crate，能力靠 feature 点单）；PostgreSQL。
- 前端：`@cailu/web` Vue 3.5 + TS + Vite 6 + TDesign Vue Next + Pinia + ECharts。
- 默认端口：后端 `8023`，前端 dev `5173`（代理 `/api`、`/health` → `:8023`）。
- 默认账号：`admin / Admin@123456`（SUPER_ADMIN 跳过权限校验）。

## 目录
```
backend/src/
  system/   support/      # 两个业务容器：每模块 mod+handler+dto+model（不设 service 层）
  middleware/ task/ utils/ shared/ bootstrap.rs
  lib.rs                    # 启动链 App::new()?...scan().start()
frontend/src/
  api/{system,support,business}/   views/{system,support,business}/
  router/index.ts  stores/  layouts/  components/
```

## 常用命令
```bash
# 后端（backend/）
cargo run                       # 自动迁移 + 启动 :8023
cargo check                     # 类型检查（零警告才提交）
cargo fmt && cargo clippy -- -D warnings

# 前端（frontend/）
pnpm install && pnpm dev        # :5173
pnpm typecheck                  # vue-tsc --noEmit

# 一键工程（项目根）
bash scripts/dev.sh             # 依赖容器 + 后端 + 前端
bash scripts/check.sh           # 提交前检查
bash scripts/build.sh           # 构建 release + 前端 dist
docker compose -f docker/docker-compose.yml up -d   # PG + Redis
```

## 必须遵守的约定
1. **后端每模块四件套** `mod.rs + handler.rs + dto.rs + model.rs`；业务编排在 `handler`，数据访问在 `model`，**不设 service 层**。
2. **二开业务放 `business/` 容器**（后端 `backend/src/business/<模块>/`、前端 `api/business/`、`views/business/<模块>/`），与 `system/`、`support/` 平级，**不侵入框架底座**。
3. **依赖铁律**：`Cargo.toml` 只声明 `alun` 一个框架 crate（`alun-utils` 必要时可加）。**不出现** `alun-db` / `alun-db-core` / `alun-web` / `alun-config`。业务代码统一 `use alun::prelude::*`。
4. **feature 语义**：默认集是 `core`（非 `full`）。用数据库必须显式 `postgres`/`mysql`/`sqlite`（`db` 只表能力、不含驱动）。预设 `mini ⊂ lite ⊂ full`；常用 `lite`（= core+postgres+cache+plugin+fs-local）。点单 feature：`fs-local`/`fs-minio`、`xlsx`、`sse`、`metrics`、`template`、`xss`。
5. **错误出口只有三个名字**：handler `ResResult<T>`、model `DbResult<T>`、启动/插件/任务 `alun::Result<T>`。handler 内 `DbError/io::Error/alun_core::Error` 直接 `?`，**不要**写多余 `.map_err(ApiError::internal(...))`。
6. **迁移号段**（动手前先 `ls backend/migrations/*/` 核对）：基础版最大 `V0017`；版本 4 位补零，已应用迁移指纹校验、不可改。
7. **权限码** `{模块}:{资源}:{动作}`，与 `sys_menu.auth_code` 和 `#[alun::permission]` 声明一一对应；`SUPER_ADMIN` 短路全部校验。
8. **路由宏路径不含 `/api` 前缀**（由 `[router] prefix` 注入）；`/health` 手动注册豁免前缀。
9. **前端 API 路径必须写全 `/api/...`**（漏前缀会命中 SPA fallback 返回 HTML 导致"响应体格式不符合"）。
10. **TDesign Grid 是 12 列制**（`t-col-xl-2`=一行 6 个、`-xl-3`=一行 4 个；`t-col-xs-24` 不存在，最大 `xs-12`）。

## 高频踩坑（先读）
- 报表/接口不显示：先查前端 api 路径是否漏 `/api`。
- 图标渲染空白：`user-check`→`user-checked`、`speed`→`rocket`（TDesign 无这两个名）。

## 详细文档（二开文档集，见 `docs/README.md` 总入口）
- `docs/README.md` — 二开文档集总入口（结构 / 设计亮点 / 前后端二开 / 编码规范 / AI 指引）
- `docs/项目结构与设计亮点.md` — 目录树、17 张表、Alun 设计亮点、部署模型
- `docs/后端二开指南.md` — 后端四件套模板、新增模块流程、迁移/事务/错误出口
- `docs/前端二开指南.md` — 前端 api/列表页/路由/主题二开
- `docs/编码规范.md` — 后端+前端编码规范与提交前检查
- `docs/AI二开指引.md` — AI 二开强制约定、反魔幻清单、范本指针
- `skill/SKILL.md` — 本仓库 AI 知识包（浓缩速查）
