//! 部门机构子模块。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysDeptModel`]（`sys_dept`） |
//! | [`dto`] | 部门树节点 / 详情 / 增改的请求与响应结构 |
//! | [`handler`] | 部门树查询、详情、增删改 |
//!
//! 对外路径前缀 `/api/system/dept`。部门以 `parent_id` 组织为树，`dept_path`
//! 冗余层级路径用于「本部门及子部门」范围查询。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/tree` | `system:dept:list` | 部门树（支持部门名模糊与状态过滤） |
//! | GET | `/{id}` | `system:dept:list` | 部门详情（不含 `children`） |
//! | POST | `/` | `system:dept:create` | 新建部门（维护 `dept_path`） |
//! | PUT | `/{id}` | `system:dept:update` | 编辑部门（重算 `dept_path`，禁止挂到自身子树） |
//! | DELETE | `/{id}` | `system:dept:delete` | 删除部门（存在子部门或归属用户时拒绝） |

pub mod dto;
pub mod handler;
pub mod model;
