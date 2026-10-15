//! 认证子模块：登录、登出、令牌刷新与当前用户信息。
//!
//! 对外路径前缀 `/api/system/auth`（见开发方案 §7.1）：
//!
//! | 接口 | 说明 | 免鉴权 |
//! |------|------|--------|
//! | `POST /api/system/auth/login` | 账密登录，返回 access / refresh 令牌 | 是 |
//! | `POST /api/system/auth/refresh` | 用 refresh 令牌换取新令牌对 | 是 |
//! | `POST /api/system/auth/logout` | 登出，令牌加入缓存黑名单 | 否 |
//! | `GET  /api/system/auth/me` | 当前登录用户信息（角色 / 权限码） | 否 |
//! | `GET  /api/system/auth/menus` | 当前登录用户可见的菜单树 | 否 |
//! | `PUT  /api/system/auth/profile` | 更新本人资料（姓名 / 邮箱 / 手机号） | 否 |
//! | `PUT  /api/system/auth/password` | 修改本人密码（校验原密码） | 否 |
//! | `GET  /api/system/auth/logs` | 本人登录（认证）日志分页 | 否 |
//!
//! # 分层
//!
//! - [`model`]：认证相关的数据访问与领域逻辑（用户 / 角色 / 权限 / 菜单查询、
//!   登录状态维护、登录安全策略），**唯一访问数据库的地方**；
//! - [`handler`]：业务编排（校验、分支、响应组装）与路由声明，不直接写 SQL；
//! - [`dto`]：请求 / 响应结构。
//!
//! 认证过程读写的 `sys_user` / `sys_role` / `sys_user_role` / `sys_menu` /
//! `sys_role_menu` / `sys_config` 均复用各自主业务模块的强类型模型，
//! 认证层不再重复定义表结构。

pub mod dto;
pub mod handler;
pub(crate) mod model;
