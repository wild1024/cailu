# 菜芦通用后台系统（Cailu）· 二开文档集

> 目标：让**开发者**用最快速度看懂项目结构、抓住设计亮点、掌握快速开发套路；让 **AI 编码 Agent** 能直接检索到「技能设计方案」与「编码规范」，生成贴合本项目约定的代码，而不是凭空魔幻生成。

本仓库是**基础版**（用户 / 角色 / 菜单 / 部门 / 字典 / 配置 / 文件 / 日志 / 在线用户 / 定时任务）。专业版 `cailu-pro` 与企业版 `cailu-saas` 为独立仓库。

---

## 一、文档地图

| 文档 | 给谁看 | 解决什么问题 |
|------|--------|--------------|
| **本文件（README）** | 所有人 | 文档总入口、5 分钟上手、核心约定速记、AI 入口 |
| [项目结构与设计亮点.md](./项目结构与设计亮点.md) | 所有人 | 完整目录树、17 张表、Alun 框架设计亮点、部署模型 |
| [后端二开指南.md](./后端二开指南.md) | 后端 / AI | 四件套模板、新增模块 7 步流程、迁移 / 事务 / 错误出口 |
| [前端二开指南.md](./前端二开指南.md) | 前端 / AI | api 封装、列表页范本、路由注册、主题变量、TDesign 坑 |
| [编码规范.md](./编码规范.md) | 所有人 / AI | 后端+前端强制规范、提交前检查命令 |
| [AI二开指引.md](./AI二开指引.md) | AI Agent | 强制铁律、反魔幻清单、二开设计方案骨架、范本指针 |
| `skill/SKILL.md`（仓库根） | AI Agent | 浓缩速查知识包（命令 / 约定 / 踩坑） |
| `AGENTS.md`（仓库根） | AI Agent | AI 操作指令与入口 |
| `llms.txt`（仓库根） | AI Agent | 文件地图（llm.txt 协议） |

> **AI 入口优先级**：动手前先读 `AGENTS.md` → `skill/SKILL.md` → 本集的 `AI二开指引.md` 与对应分册，优先于逐文件检索，可显著节省 token 并避免误判。

---

## 二、5 分钟上手

### 技术栈速览
- **后端**：Rust 1.94.1+ / edition 2024，框架 `alun`（门面，底层 axum 0.8），PostgreSQL ≥ 14，Redis（缓存 / 令牌黑名单）。
- **前端**：Vue 3.5 + TS 5.8 + Vite 6 + TDesign Vue Next 1.19 + Pinia 3 + ECharts 5，pnpm 10.5。
- **端口**：后端 `8023`；前端 dev `5173`（Vite 代理 `/api`、`/health` → `:8023`）。
- **默认账号**：`admin / Admin@123456`（`SUPER_ADMIN` 跳过全部权限校验）。

### 一条命令起环境
```bash
bash scripts/dev.sh          # 依赖容器(PG+Redis) + 后端 + 前端，一键拉起
```
或分别：
```bash
cd backend && cargo run                       # 自动迁移 + 启动 :8023
cd frontend && pnpm install && pnpm dev       # :5173
```

### 提交前检查
```bash
cd backend && cargo fmt && cargo clippy -- -D warnings && cargo check
cd frontend && pnpm typecheck
# 或： bash scripts/check.sh
```

---

## 三、核心约定速记（必须刻进肌肉记忆）

1. **后端每模块四件套**：`mod.rs + handler.rs + dto.rs + model.rs`；业务编排在 `handler`，数据访问与领域行为在 `model`，**不设 service 层**。
2. **二开业务放 `business/` 容器**：后端 `src/business/<模块>/`、前端 `api/business/`、`views/business/<模块>/`，与 `system/`、`support/` 平级，**不侵入框架底座**。
3. **依赖铁律**：`Cargo.toml` 只声明 `alun`（必要时 `alun-utils`）；**禁止**写 `alun-db` / `alun-db-core` / `alun-web` / `alun-config`。业务代码统一 `use alun::prelude::*`。
4. **错误出口三名字**：handler `ResResult<T>`、model `DbResult<T>`、启动/插件/任务 `alun::Result<T>`；handler 内 `?` 直接传播，不要写多余 `.map_err(ApiError::internal(...))`。
5. **迁移号段**：基础版最大 `V0017`；二开业务从 **`V0020`** 起编号（留 `V0018-19` 缓冲），4 位补零，已应用迁移指纹校验不可改。
6. **权限码三段式** `{模块}:{资源}:{动作}`（如 `system:user:create`），与 `sys_menu.auth_code` 和 `#[alun::permission(...)]` 一一对应；`SUPER_ADMIN` 短路。
7. **路由宏路径不含 `/api` 前缀**（由 `[router] prefix` 注入）；`/health` 手动注册免前缀。
8. **前端 API 路径必须写全 `/api/...`**（漏前缀命中 SPA fallback 返回 HTML → “响应体格式不符合”）。
9. **TDesign Grid 是 12 列制**：`t-col-xl-2`=一行 6 个、`-xl-3`=一行 4 个；`t-col-xs-24` 不存在（最大 `xs-12`）。

---

## 四、给 AI 的特别说明

- 本项目已为 AI 准备好 `AGENTS.md`、`llms.txt`、`skill/SKILL.md` 与本文档集。**不要跳过它们凭印象生成代码**——尤其是「依赖只写 alun」「四件套结构」「错误出口三名字」「迁移号段 V0020+」「权限码三段式」「前端 /api 前缀」这些铁律。
- 具体反魔幻清单、二开设计方案骨架见 [AI二开指引.md](./AI二开指引.md)。
- 范本永远优先照抄：`backend/src/system/user/` 是后端四件套范本；`frontend/src/views/system/user/index.vue` + `frontend/src/api/system/user-api.ts` 是前端列表页范本。
