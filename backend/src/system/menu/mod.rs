//! 菜单与权限点子模块。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysMenuModel`]（`sys_menu`）、[`model::MenuType`]（菜单类型枚举） |
//!
//! 对外路径前缀 `/api/system/menu`。菜单以 `parent_id` 组织为树，`auth_code`
//! 承载权限码（`{模块}:{资源}:{动作}`）。

pub mod dto;
pub mod handler;
pub mod model;
