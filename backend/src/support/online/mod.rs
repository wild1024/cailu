//! 在线用户子模块：登录令牌维度的在线列表与强制下线。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::OnlineSession`] 与缓存数据访问 [`model::OnlineSessionModel`] |
//! | [`dto`] | 请求 / 响应结构（[`dto::OnlineRes`] 等） |
//! | [`handler`] | 在线列表与强制下线接口 |
//!
//! 会话由登录 / 刷新成功时登记（见 [`crate::system::auth::handler`]），
//! 登出时移除；不落数据库，完全由缓存承载。
//!
//! # 接口清单
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/api/support/online/page` | `support:online:list` | 在线用户分页（`page` / `page_size` / `username?`） |
//! | DELETE | `/api/support/online/{token_id}` | `support:online:force` | 强制下线（令牌 `jti`） |

pub mod dto;
pub mod handler;
pub mod model;
