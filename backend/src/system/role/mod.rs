//! 角色管理子模块：角色与「角色-菜单」授权。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysRoleModel`]（`sys_role`）、[`model::SysRoleMenuModel`]（`sys_role_menu`） |
//! | [`dto`] | 角色列表 / 增改 / 菜单授权的请求与响应结构 |
//! | [`handler`] | 角色分页、下拉、详情、增删改与菜单授权 |
//!
//! 对外路径前缀 `/api/system/role`。`sys_role_menu` 为硬删关联表，重新授权
//! 即整批重建记录；内置角色 `SUPER_ADMIN` 禁止删除与禁用。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/page` | `system:role:list` | 角色分页（支持角色名 / 编码模糊与状态过滤） |
//! | GET | `/list` | `system:role:list` | 全部启用角色（按排序值升序，供下拉） |
//! | GET | `/{id}` | `system:role:list` | 角色详情 |
//! | POST | `/` | `system:role:create` | 新建角色 |
//! | PUT | `/{id}` | `system:role:update` | 编辑角色（全量业务字段） |
//! | DELETE | `/{id}` | `system:role:delete` | 删除角色（内置或已分配用户的角色不可删） |
//! | GET | `/{id}/menus` | `system:role:list` | 查询已授权菜单 id 列表 |
//! | PUT | `/{id}/menus` | `system:role:menu` | 保存菜单授权（先清空再逐条写入） |

pub mod dto;
pub mod handler;
pub mod model;
