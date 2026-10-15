//! 认证模块的请求与响应 DTO。

use serde::{Deserialize, Serialize};
use validator::Validate;

/// 账密登录请求。
#[derive(Debug, Deserialize, Validate)]
pub struct LoginReq {
    /// 登录账号。
    #[validate(length(min = 1, max = 64, message = "用户名长度需为 1-64 个字符"))]
    pub username: String,
    /// 登录密码（明文，仅在传输与校验瞬间存在）。
    #[validate(length(min = 1, max = 128, message = "密码长度需为 1-128 个字符"))]
    pub password: String,
}

/// 刷新令牌请求。
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenReq {
    /// 上一步签发的 refresh token。
    #[validate(length(min = 1, message = "refresh_token 不能为空"))]
    pub refresh_token: String,
}

/// 登录用户摘要（嵌入令牌响应）。
#[derive(Debug, Clone, Serialize)]
pub struct LoginUserRes {
    /// 用户 ID（UUID 字符串）。
    pub id: String,
    /// 登录账号。
    pub username: String,
    /// 真实姓名。
    pub real_name: Option<String>,
}

/// 令牌响应。
///
/// `user` 仅在登录 / 刷新时下发；续期接口复用同一结构时可通过 `None` 省略。
#[derive(Debug, Serialize)]
pub struct TokenRes {
    /// 访问令牌。
    pub access_token: String,
    /// 刷新令牌。
    pub refresh_token: String,
    /// 令牌类型，固定 `Bearer`。
    pub token_type: String,
    /// 访问令牌有效期（秒）。
    pub expires_in: u64,
    /// 登录用户摘要（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<LoginUserRes>,
}

/// 更新当前登录用户个人资料的请求。
///
/// 字段均为可选：为 `None` 时保持原值不变，提交空串则清空该字段。
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileReq {
    /// 真实姓名。
    #[validate(length(min = 1, max = 64, message = "姓名长度需为 1-64 个字符"))]
    pub real_name: Option<String>,
    /// 电子邮箱。
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
    /// 手机号。
    #[validate(length(max = 20, message = "手机号长度不能超过 20 个字符"))]
    pub mobile: Option<String>,
}

/// 修改当前登录用户密码的请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordReq {
    /// 原密码（用于二次校验身份）。
    #[validate(length(min = 1, max = 128, message = "原密码不能为空"))]
    pub old_password: String,
    /// 新密码。
    #[validate(length(min = 6, max = 128, message = "新密码长度需为 6-128 个字符"))]
    pub new_password: String,
}

/// 当前登录用户信息响应。
#[derive(Debug, Serialize)]
pub struct CurrentUserRes {
    /// 用户 ID。
    pub id: String,
    /// 登录账号。
    pub username: String,
    /// 真实姓名。
    pub real_name: Option<String>,
    /// 电子邮箱。
    pub email: Option<String>,
    /// 手机号。
    pub mobile: Option<String>,
    /// 所属部门 ID。
    pub dept_id: Option<String>,
    /// 角色编码列表。
    pub roles: Vec<String>,
    /// 权限码列表（超级管理员为 `["*:*:*"]`）。
    pub permissions: Vec<String>,
}
