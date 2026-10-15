//! 角色管理的请求与响应 DTO。
//!
//! 字段命名与前端契约 `frontend/src/api/system/role-api.ts` 严格对齐：可空文本列
//! （`description` / `remark`）映射时统一下发为空串，`created_at` 保留原值下发。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::system::role::model::SysRoleModel;

/// 角色列表项响应（分页、下拉与详情共用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRes {
    /// 角色 id
    pub id: String,
    /// 角色编码
    pub role_code: String,
    /// 角色名称
    pub role_name: String,
    /// 角色描述
    pub description: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 排序值
    pub sort_order: i32,
    /// 备注
    pub remark: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&SysRoleModel> for RoleRes {
    /// 由角色模型构造响应。
    ///
    /// # 参数
    ///
    /// - `m`：角色模型引用。
    ///
    /// # 返回值
    ///
    /// 角色响应对象；可空文本列统一降级为空串。
    fn from(m: &SysRoleModel) -> Self {
        Self {
            id: m.id.to_string(),
            role_code: m.role_code.clone(),
            role_name: m.role_name.clone(),
            description: m.description.clone().unwrap_or_default(),
            status: m.status.as_i16(),
            sort_order: m.sort_order,
            remark: m.remark.clone().unwrap_or_default(),
            created_at: m.created_at,
        }
    }
}

/// 角色分页查询参数（全部可选）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RolePageQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页条数
    pub page_size: Option<u64>,
    /// 角色名称（模糊）
    pub role_name: Option<String>,
    /// 角色编码（模糊）
    pub role_code: Option<String>,
    /// 状态（0 停用 / 1 正常）
    pub status: Option<i16>,
}

/// 新增 / 编辑角色请求（PUT 全量更新，字段一致）。
#[derive(Debug, Deserialize, Validate)]
pub struct RoleSaveReq {
    /// 角色编码
    #[validate(length(min = 1, max = 64, message = "角色编码长度需为 1-64 个字符"))]
    pub role_code: String,
    /// 角色名称
    #[validate(length(min = 1, max = 64, message = "角色名称长度需为 1-64 个字符"))]
    pub role_name: String,
    /// 角色描述
    #[validate(length(max = 255, message = "角色描述长度不能超过 255 个字符"))]
    pub description: Option<String>,
    /// 状态（0 停用 / 1 正常），缺省启用
    pub status: Option<i16>,
    /// 排序值
    pub sort_order: Option<i32>,
    /// 备注
    #[validate(length(max = 255, message = "备注长度不能超过 255 个字符"))]
    pub remark: Option<String>,
}

/// 保存角色菜单授权请求。
#[derive(Debug, Deserialize, Validate)]
pub struct RoleMenuAssignReq {
    /// 菜单 id 列表；空数组表示清空全部授权
    pub menu_ids: Vec<String>,
}

/// 角色用户列表分页查询参数（全部可选，缺省取框架默认分页）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoleUserPageQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页条数
    pub page_size: Option<u64>,
}

/// 角色用户列表项响应（角色管理右侧「角色用户列表」页签）。
#[derive(Debug, Clone, Serialize)]
pub struct RoleUserRes {
    /// 用户 id
    pub id: String,
    /// 用户名
    pub username: String,
    /// 真实姓名
    pub real_name: String,
    /// 手机号
    pub mobile: String,
    /// 状态（0 停用 / 1 正常）
    pub status: i16,
    /// 该用户拥有的角色名称列表
    pub roles: Vec<String>,
}
