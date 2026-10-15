//! 用户管理的请求与响应 DTO。
//!
//! 字段命名与前端契约 `frontend/src/api/system/user-api.ts` 严格对齐：数据库中的可空文本列
//! （`real_name` / `email` / `mobile` / 部门名 / `remark`）映射时统一下发为空串，
//! 时间字段保留可空下发（前端自行渲染为 `-`）。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::system::user::model::SysUserModel;

/// 用户关联角色的简要信息（列表 / 详情内嵌）。
#[derive(Debug, Clone, Serialize)]
pub struct RoleBriefRes {
    /// 角色 id
    pub role_id: String,
    /// 角色编码
    pub role_code: String,
    /// 角色名称
    pub role_name: String,
}

/// 用户列表项响应。
#[derive(Debug, Clone, Serialize)]
pub struct UserRes {
    /// 用户 id
    pub id: String,
    /// 用户名
    pub username: String,
    /// 真实姓名
    pub real_name: String,
    /// 邮箱
    pub email: String,
    /// 手机号
    pub mobile: String,
    /// 所属部门 id
    pub dept_id: Option<String>,
    /// 所属部门名称
    pub dept_name: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 是否被锁定
    pub is_locked: bool,
    /// 最近登录时间
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 关联角色简要列表
    pub roles: Vec<RoleBriefRes>,
}

impl UserRes {
    /// 由用户模型与部门名称构造列表项（`roles` 初始为空，由处理器聚合后填充）。
    ///
    /// # 参数
    ///
    /// - `m`：用户模型引用。
    /// - `dept_name`：所属部门名称；无部门或部门已删除时传空串。
    ///
    /// # 返回值
    ///
    /// 用户列表项响应；可空文本列统一降级为空串。
    pub fn from_model(m: &SysUserModel, dept_name: String) -> Self {
        Self {
            id: m.id.to_string(),
            username: m.username.clone(),
            real_name: m.real_name.clone().unwrap_or_default(),
            email: m.email.clone().unwrap_or_default(),
            mobile: m.mobile.clone().unwrap_or_default(),
            dept_id: m.dept_id.map(|v| v.to_string()),
            dept_name,
            status: m.status.as_i16(),
            is_locked: m.is_locked,
            last_login_at: m.last_login_at,
            created_at: m.created_at,
            roles: Vec::new(),
        }
    }
}

/// 用户详情响应（比列表项多备注）。
#[derive(Debug, Clone, Serialize)]
pub struct UserDetailRes {
    /// 用户 id
    pub id: String,
    /// 用户名
    pub username: String,
    /// 真实姓名
    pub real_name: String,
    /// 邮箱
    pub email: String,
    /// 手机号
    pub mobile: String,
    /// 所属部门 id
    pub dept_id: Option<String>,
    /// 所属部门名称
    pub dept_name: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 是否被锁定
    pub is_locked: bool,
    /// 最近登录时间
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 备注
    pub remark: String,
    /// 关联角色简要列表
    pub roles: Vec<RoleBriefRes>,
}

impl UserDetailRes {
    /// 由用户模型与部门名称构造详情（`roles` 初始为空，由处理器聚合后填充）。
    ///
    /// # 参数
    ///
    /// - `m`：用户模型引用。
    /// - `dept_name`：所属部门名称；无部门或部门已删除时传空串。
    ///
    /// # 返回值
    ///
    /// 用户详情响应；可空文本列统一降级为空串。
    pub fn from_model(m: &SysUserModel, dept_name: String) -> Self {
        let base = UserRes::from_model(m, dept_name);
        Self {
            id: base.id,
            username: base.username,
            real_name: base.real_name,
            email: base.email,
            mobile: base.mobile,
            dept_id: base.dept_id,
            dept_name: base.dept_name,
            status: base.status,
            is_locked: base.is_locked,
            last_login_at: base.last_login_at,
            created_at: base.created_at,
            remark: m.remark.clone().unwrap_or_default(),
            roles: Vec::new(),
        }
    }
}

/// 用户分页查询参数（全部可选）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserPageQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页条数
    pub page_size: Option<u64>,
    /// 用户名（模糊）
    pub username: Option<String>,
    /// 真实姓名（模糊）
    pub real_name: Option<String>,
    /// 邮箱（模糊）
    pub email: Option<String>,
    /// 手机号（模糊）
    pub mobile: Option<String>,
    /// 状态（0 停用 / 1 正常）
    pub status: Option<i16>,
    /// 部门 id
    pub dept_id: Option<String>,
    /// 是否包含所选部门的子部门用户（true 时按部门子树过滤）
    pub include_children: Option<bool>,
    /// 创建时间起始（含，ISO 时间字符串）
    pub create_begin: Option<chrono::DateTime<chrono::Utc>>,
    /// 创建时间截止（含，ISO 时间字符串）
    pub create_end: Option<chrono::DateTime<chrono::Utc>>,
}

/// 新增用户请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UserCreateReq {
    /// 用户名
    #[validate(length(min = 1, max = 64, message = "用户名长度需为 1-64 个字符"))]
    pub username: String,
    /// 初始密码（明文，服务端哈希后存储）
    #[validate(length(min = 6, max = 64, message = "密码长度需为 6-64 个字符"))]
    pub password: String,
    /// 真实姓名
    #[validate(length(max = 64, message = "真实姓名长度不能超过 64 个字符"))]
    pub real_name: Option<String>,
    /// 邮箱
    #[validate(length(max = 128, message = "邮箱长度不能超过 128 个字符"))]
    pub email: Option<String>,
    /// 手机号
    #[validate(length(max = 32, message = "手机号长度不能超过 32 个字符"))]
    pub mobile: Option<String>,
    /// 所属部门 id
    pub dept_id: Option<String>,
    /// 状态（0 停用 / 1 正常），缺省启用
    pub status: Option<i16>,
    /// 备注
    #[validate(length(max = 255, message = "备注长度不能超过 255 个字符"))]
    pub remark: Option<String>,
    /// 角色 id 列表（创建用户同时分配角色，可为空，缺省为空）
    #[serde(default)]
    pub role_ids: Vec<String>,
}

/// 编辑用户请求（用户名与密码不在编辑范围内）。
#[derive(Debug, Deserialize, Validate)]
pub struct UserUpdateReq {
    /// 真实姓名
    #[validate(length(max = 64, message = "真实姓名长度不能超过 64 个字符"))]
    pub real_name: Option<String>,
    /// 邮箱
    #[validate(length(max = 128, message = "邮箱长度不能超过 128 个字符"))]
    pub email: Option<String>,
    /// 手机号
    #[validate(length(max = 32, message = "手机号长度不能超过 32 个字符"))]
    pub mobile: Option<String>,
    /// 所属部门 id
    pub dept_id: Option<String>,
    /// 状态（0 停用 / 1 正常）
    pub status: Option<i16>,
    /// 备注
    #[validate(length(max = 255, message = "备注长度不能超过 255 个字符"))]
    pub remark: Option<String>,
}

/// 修改用户状态请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UserStatusReq {
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
}

/// 重置用户密码请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UserPasswordReq {
    /// 新密码（明文，服务端哈希后存储）
    #[validate(length(min = 6, max = 64, message = "密码长度需为 6-64 个字符"))]
    pub password: String,
}

/// 分配用户角色请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UserRoleAssignReq {
    /// 角色 id 列表；空数组表示清空全部角色
    pub role_ids: Vec<String>,
}

/// 批量调整用户部门请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UserBatchDeptReq {
    /// 待调整的用户 id 列表（至少 1 个）
    #[validate(length(min = 1, message = "请至少选择一个用户"))]
    pub user_ids: Vec<String>,
    /// 目标部门 id（必填，统一调入该部门）
    #[validate(length(min = 1, message = "目标部门不能为空"))]
    pub dept_id: String,
}
