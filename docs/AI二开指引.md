# AI 二开指引（防止魔幻生成）

> 本文档专门给 **AI 编码 Agent** 看。目的：让 AI 在 cailu 基础版上二开时，能检索到「技能设计方案」与「编码规范」，生成**贴合本项目约定**的代码，而不是凭通用模板魔幻生成（错依赖、错分层、错错误出口、漏 `/api` 前缀等）。
> 入口优先级：`AGENTS.md` → `skill/SKILL.md` → 本文件 → 各分册。动代码前先读完。

---

## 一、先读什么（不要跳过）

1. `AGENTS.md`：操作指令 + 核心约定 10 条。
2. `skill/SKILL.md`：浓缩速查（命令 / 约定 / 踩坑）。
3. 本文件：强制铁律 + 反魔幻清单 + 二开设计方案骨架。
4. 分册：[项目结构与设计亮点.md](./项目结构与设计亮点.md) / [后端二开指南.md](./后端二开指南.md) / [前端二开指南.md](./前端二开指南.md) / [编码规范.md](./编码规范.md)。

---

## 二、强制铁律（违反任意一条 = 生成的代码不可用）

| # | 铁律 | 错误示例（魔幻生成） | 正确做法 |
|---|------|----------------------|----------|
| 1 | `Cargo.toml` 只声明 `alun`（必要时 `alun-utils`） | `cargo add alun-db` / `use alun_db::...` | feature 点单：`features = ["postgres", "cache", ...]`；`use alun::prelude::*` |
| 2 | 后端每模块四件套 `mod/handler/dto/model`，**不设 service 层** | 新建 `service.rs` 转发 | handler 编排、model 数据访问 |
| 3 | 二开业务放 `business/`，不侵入 `system/`/`support/` | 把业务塞进 `system/user/` | `src/business/<模块>/` + `api/business/` + `views/business/` |
| 4 | 错误出口三名字；handler 内 `?` 直接传播 | `map_err(ApiError::internal(...))` 到处写 | `?` 或 `utils::db_error(e, "ctx")` |
| 5 | 迁移号段：基础版封顶 `V0017`，二开从 **`V0020`** 起；指纹不可改 | 新建 `V0018` 或回改历史迁移 | 新版本 `V0020__*.up.sql` |
| 6 | 权限码三段式 `{模块}:{资源}:{动作}`，三处一致 | 随意命名 `article_manage` | `business:article:create` 且 `sys_menu.auth_code` 同值 |
| 7 | 路由宏路径**不含** `/api` 前缀 | `#[alun::get("/api/business/article")]` | `#[alun::get("/business/article")]` |
| 8 | 前端 API 路径**写全** `/api/...` | `get('business/article/page')` | `get('/api/business/article/page', ...)` |
| 9 | 禁止 `unwrap()` / `expect()` | `uuid.parse().unwrap()` | `utils::require_uuid(&s, "id")?` |
| 10 | SQL 用 `sql!` 宏，绝不字符串拼接 | `format!("WHERE x='{}'", v)` | `#if / #and` 自动参数化 |

---

## 三、二开设计方案（照方抓药：新增一个 CRUD 模块）

> 当任务为「新增业务模块 / 实体」时，严格按此骨架生成，不要另起炉灶。

**后端（放 `src/business/<模块>/`）**：
1. `migrations/business/V0020__create_<表>.up.sql`（三段式：建表 → COMMENT ON → 索引；`.down.sql` 成对）。
2. `model.rs`：`#[derive(Model)] #[table("biz_xxx")]`；实体字段 + 需自写 SQL 的访问函数。
3. `dto.rs`：`*Req`（带 `validator::Validate`）、`*Res`（带 `from_model`，可空文本列降级空串）。
4. `handler.rs`：`#[alun::permission(path, method, permission)]` + `#[alun::get/post/put/patch/delete(...)]`；返回 `ResResult<T>`，用 `Res::ok` / `Res::page`。
5. `mod.rs`：`pub mod dto; handler; model;` + 模块文档与接口一览表。
6. `migrations/seed/`：追加菜单 + 按钮权限点（`module_code='business'`，`ON CONFLICT ... WHERE deleted_at IS NULL` 幂等）。

**前端（放 `views/business/<模块>/` + `api/business/`）**：
1. `api/business/<模块>-api.ts`：方法调 `request.ts` 的 `get/post/put/patch/del`，路径写全 `/api/...`。
2. `views/business/<模块>/index.vue`：搜索表单 + `t-table` + 分页 + 新增/编辑抽屉（插槽用 `body`，非 `content`）。
3. `router/index.ts`：在 `BasicLayout` children 加一条，`meta: { title, permission }`，懒加载 `import('@/views/business/<模块>/index.vue')`。
4. 类型命名与后端 DTO 对齐；样式用 `--app-*` 主题变量。

**验证**：`cargo fmt && cargo clippy -- -D warnings && cargo check`；`pnpm typecheck`。

---

## 四、反魔幻清单（DO / DON'T）

**DO**
- 改依赖只动 `features = [...]`（如加 `xlsx`、`sse`），绝不 `cargo add alun-*`。
- 模型用 `#[derive(Model)]`，查询用 `SysXxxModel::query() / find_by / paginate`，写入用实例方法 `.insert/.update/.delete`。
- 事务用 `db().transaction(|tx| async move { ... })`。
- 当前用户用 `Extension(AuthClaims(claims))` 取，审计字段填 `claims.user_id`。
- 分页响应用 `Res::page(list, total, page, size)`；前端 `PageData<T>` 取 `list/total/page/page_size`。
- 字典 / 部门 / 文件复用 `components/` 的 `DictSelect` / `DeptTreeSelect` / `FileUploader`。

**DON'T**
- 不要把后端 service 层、DTO 映射层、Repository 层塞进来（本项目刻意扁平）。
- 不要把 `system/` / `support/` 底座当业务目录改（二开放 `business/`）。
- 不要在 handler 里手写 SQL 字符串或裸 `sqlx::query`（用 `sql!` 宏）。
- 不要漏 `/api` 前缀（前端）或写死 `/api` 前缀（后端宏）。
- 不要写 `unwrap/expect`、不要静默写空串、不要 `tmp/var/data` 命名。
- 不要新建 `V0018` 或改动已应用迁移；不要写 `t-col-xs-24`、图标用 `user-check`/`speed`。

---

## 五、范本文件指针（生成前先读）

| 想生成什么 | 照抄哪个范本 |
|------------|--------------|
| 后端模块四件套 | `backend/src/system/user/{mod,handler,dto,model}.rs` |
| 后端事务 / 批量 | `backend/src/system/user/handler.rs` 的 `create`（含 `db().transaction`） |
| 后端鉴权 / 登录 / 限流 | `backend/src/system/auth/handler.rs`、`model.rs` |
| 前端列表页 | `frontend/src/views/system/user/index.vue` |
| 前端接口封装 | `frontend/src/api/system/user-api.ts` |
| 前端路由 / 守卫 | `frontend/src/router/index.ts` |
| 前端 axios 封装 | `frontend/src/api/request.ts` |
| 主题 / 渐变变量 | `frontend/src/style/theme.less` |
| 启动链 / 分层 | `backend/src/lib.rs` |
| 配置 / 部署 | `backend/config/config.toml`、`docker/`、`scripts/` |
| 框架依赖（crates.io 发布版） | `alun = { version = "0.3.0" }`（见 `backend/Cargo.toml`；构建时由 cargo 从 crates.io 拉取，无需本地框架源码） |

---

## 六、常见误生成 → 纠正

- **误**：`let id = uuid::Uuid::parse_str(&s).unwrap();` → **正**：`let id = utils::require_uuid(&s, "id")?;`
- **误**：`#[alun::post("/api/business/x")]` → **正**：`#[alun::post("/business/x")]`（前缀由配置注入）。
- **误**：`get('business/x/page')` → **正**：`get('/api/business/x/page', params)`。
- **误**：新建 `service/business_x.rs` → **正**：`business/x/handler.rs` + `model.rs`。
- **误**：`cargo add sqlx` → **正**：在 `alun` 的 `features` 加 `"postgres"`。
- **误**：`V0018__xxx.sql` → **正**：`V0020__xxx.sql`（留 `V0018-19` 缓冲，且基础版封顶 `V0017`）。
- **误**：抽屉内容写 `#content` 槽 → **正**：`t-drawer` 用 `#body` / 默认槽。

---

## 七、二开前必做

- 改迁移前先 `ls backend/migrations/*/` 核对当前最大版本号，确认号段。
- 新增权限点后，确认 `sys_menu.auth_code` 种子、`#[alun::permission(...)]`、`router.meta.permission` 三处字符串完全一致。
- 提交前跑 `scripts/check.sh`（后端 fmt/clippy/check + 前端 typecheck）。
