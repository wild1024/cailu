//! 部门机构的请求与响应 DTO。
//!
//! 字段命名与前端契约 `frontend/src/api/system/dept-api.ts` 严格对齐：可空文本列
//! （`phone` / `email`）映射时统一下发为空串；树节点在列表场景递归填充
//! `children`，详情场景不含该字段。

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::system::dept::model::SysDeptModel;

/// 部门树节点响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeptNode {
    /// 部门 id
    pub id: String,
    /// 部门编码
    pub dept_code: String,
    /// 部门名称
    pub dept_name: String,
    /// 上级部门 id
    pub parent_id: Option<String>,
    /// 负责人用户 id
    pub leader_id: Option<String>,
    /// 负责人姓名（`real_name` 优先，缺省回退 `username`；无负责人或用户已删除时为 `None`）
    pub leader_name: Option<String>,
    /// 联系电话
    pub phone: String,
    /// 邮箱
    pub email: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 排序值
    pub sort_order: i32,
    /// 子部门
    pub children: Vec<DeptNode>,
}

impl From<&SysDeptModel> for DeptNode {
    /// 由部门模型构造树节点（`children` 初始为空）。
    ///
    /// # 参数
    ///
    /// - `m`：部门模型引用。
    ///
    /// # 返回值
    ///
    /// 部门树节点；可空文本列统一降级为空串。
    fn from(m: &SysDeptModel) -> Self {
        Self {
            id: m.id.to_string(),
            dept_code: m.dept_code.clone(),
            dept_name: m.dept_name.clone(),
            parent_id: m.parent_id.map(|v| v.to_string()),
            leader_id: m.leader_id.map(|v| v.to_string()),
            // 姓名由处理器在树查询时批量填充（避免逐部门 N+1 查询）
            leader_name: None,
            phone: m.phone.clone().unwrap_or_default(),
            email: m.email.clone().unwrap_or_default(),
            status: m.status.as_i16(),
            sort_order: m.sort_order,
            children: Vec::new(),
        }
    }
}

/// 部门详情响应（不含 `children`）。
#[derive(Debug, Clone, Serialize)]
pub struct DeptDetailRes {
    /// 部门 id
    pub id: String,
    /// 部门编码
    pub dept_code: String,
    /// 部门名称
    pub dept_name: String,
    /// 上级部门 id
    pub parent_id: Option<String>,
    /// 负责人用户 id
    pub leader_id: Option<String>,
    /// 联系电话
    pub phone: String,
    /// 邮箱
    pub email: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 排序值
    pub sort_order: i32,
}

impl From<&SysDeptModel> for DeptDetailRes {
    /// 由部门模型构造详情。
    ///
    /// # 参数
    ///
    /// - `m`：部门模型引用。
    ///
    /// # 返回值
    ///
    /// 部门详情响应；可空文本列统一降级为空串。
    fn from(m: &SysDeptModel) -> Self {
        Self {
            id: m.id.to_string(),
            dept_code: m.dept_code.clone(),
            dept_name: m.dept_name.clone(),
            parent_id: m.parent_id.map(|v| v.to_string()),
            leader_id: m.leader_id.map(|v| v.to_string()),
            phone: m.phone.clone().unwrap_or_default(),
            email: m.email.clone().unwrap_or_default(),
            status: m.status.as_i16(),
            sort_order: m.sort_order,
        }
    }
}

/// 部门树查询参数（全部可选）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DeptTreeQuery {
    /// 部门名称（模糊）
    pub dept_name: Option<String>,
    /// 状态（0 停用 / 1 正常）
    pub status: Option<i16>,
}

/// 新增 / 编辑部门请求（PUT 全量更新，字段一致）。
#[derive(Debug, Deserialize, Validate)]
pub struct DeptSaveReq {
    /// 部门编码
    #[validate(length(min = 1, max = 64, message = "部门编码长度需为 1-64 个字符"))]
    pub dept_code: String,
    /// 部门名称
    #[validate(length(min = 1, max = 64, message = "部门名称长度需为 1-64 个字符"))]
    pub dept_name: String,
    /// 上级部门 id（为空表示根部门）
    pub parent_id: Option<String>,
    /// 负责人用户 id
    pub leader_id: Option<String>,
    /// 联系电话
    #[validate(length(max = 32, message = "联系电话长度不能超过 32 个字符"))]
    pub phone: Option<String>,
    /// 邮箱
    #[validate(length(max = 128, message = "邮箱长度不能超过 128 个字符"))]
    pub email: Option<String>,
    /// 状态（0 停用 / 1 正常），缺省启用
    pub status: Option<i16>,
    /// 排序值
    pub sort_order: Option<i32>,
}

/// 迁移部门用户并删除部门请求。
///
/// 删除部门前，把该部门下的用户统一迁移到目标部门；目标部门为空表示把这些用户
/// 移出部门（`dept_id` 置空）。
#[derive(Debug, Deserialize, Validate)]
pub struct DeptMigrateReq {
    /// 目标部门 id（用户迁入的部门）；为空表示迁出至「无部门」
    pub migrate_to_dept_id: Option<String>,
}
