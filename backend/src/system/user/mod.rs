//! 用户管理子模块：用户账号与「用户-角色」关联。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysUserModel`]（`sys_user`）、[`model::SysUserRoleModel`]（`sys_user_role`） |
//! | [`dto`] | 用户列表 / 详情 / 增改 / 状态 / 密码 / 角色分配的请求与响应结构 |
//! | [`handler`] | 用户分页、详情、增删改、启停、重置密码与角色分配 |
//!
//! 对外路径前缀 `/api/system/user`。`sys_user_role` 为硬删关联表，重新分配角色
//! 即整批重建记录。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/page` | `system:user:list` | 用户分页（支持用户名 / 姓名 / 状态 / 部门过滤，附带部门名） |
//! | GET | `/{id}` | `system:user:list` | 用户详情 |
//! | POST | `/` | `system:user:create` | 新建用户 |
//! | PUT | `/{id}` | `system:user:update` | 编辑用户（全量业务字段） |
//! | DELETE | `/{id}` | `system:user:delete` | 删除用户（同时清理角色绑定；内置管理员不可删） |
//! | PATCH | `/{id}/status` | `system:user:status` | 启用 / 停用 |
//! | PATCH | `/{id}/password` | `system:user:password` | 重置密码 |
//! | GET | `/{id}/roles` | `system:user:list` | 查询已分配角色 id 列表 |
//! | PUT | `/{id}/roles` | `system:user:role` | 分配角色（先清空再逐条写入） |

pub mod dto;
pub mod handler;
pub mod model;
