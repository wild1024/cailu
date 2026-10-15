//! 系统参数的请求与响应 DTO。
//!
//! 字段命名与前端契约 `frontend/src/api/support/config-api.ts` 严格对齐：可空列在响应中统一归一为
//! 非空字符串（`unwrap_or_default()`），时间字段按 ISO 8601 下发。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::support::config::model;
use crate::support::config::model::SysConfigModel;

/// 系统参数响应。
#[derive(Debug, Clone, Serialize)]
pub struct ConfigRes {
    /// 配置 id。
    pub id: String,
    /// 配置键。
    pub config_key: String,
    /// 配置值。
    pub config_value: String,
    /// 值类型（`STRING` / `INT` / `FLOAT` / `BOOL` / `JSON` 等）。
    pub config_type: String,
    /// 配置说明。
    pub description: String,
    /// 配置分类。
    pub category: String,
    /// 排序值。
    pub sort_order: i32,
    /// 配置级别，数值越小优先级越高。
    pub config_level: i32,
    /// 是否允许在界面修改。
    pub is_modifiable: bool,
    /// 是否加密存储。
    pub is_encrypted: bool,
    /// 是否需重启生效。
    pub is_restart: bool,
    /// 状态（1 启用 / 0 禁用）。
    pub status: i16,
    /// 引用值（其他配置键）。
    pub ref_value: String,
    /// 可选值列表。
    pub select_value: String,
    /// 创建时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ConfigRes {
    /// 由系统参数模型构造响应。
    ///
    /// 加密存储的参数值在此解密后下发，保证前端拿到的是明文。
    ///
    /// # 参数
    ///
    /// - `m`：系统参数模型引用。
    ///
    /// # 返回值
    ///
    /// 解密成功返回对应的系统参数响应对象。
    ///
    /// # 异常情况
    ///
    /// 加密参数解密失败时返回 500 错误（错误码见
    /// [`crate::shared::error_code::CONFIG_DECRYPT_FAILED`]）。
    pub(crate) fn from_model(m: &SysConfigModel) -> std::result::Result<Self, ApiError> {
        let config_value = match m.config_value.as_deref() {
            Some(raw) if m.is_encrypted => model::decrypt_config_value(raw)?,
            Some(raw) => raw.to_string(),
            None => String::new(),
        };
        Ok(Self {
            id: m.id.to_string(),
            config_key: m.config_key.clone(),
            config_value,
            config_type: m.config_type.clone(),
            description: m.description.clone().unwrap_or_default(),
            category: m.category.clone().unwrap_or_default(),
            sort_order: m.sort_order,
            config_level: i32::from(m.config_level),
            is_modifiable: m.is_modifiable,
            is_encrypted: m.is_encrypted,
            is_restart: m.is_restart,
            status: m.status.as_i16(),
            ref_value: m.ref_value.clone().unwrap_or_default(),
            select_value: m.select_value.clone().unwrap_or_default(),
            created_at: m.created_at,
        })
    }
}

/// 系统参数分页查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigPageQuery {
    /// 页码，从 1 开始（缺省为 1）。
    pub page: Option<u64>,
    /// 每页条数（缺省为框架默认值，超出上限会被规整）。
    pub page_size: Option<u64>,
    /// 配置键（模糊匹配）。
    pub config_key: Option<String>,
    /// 值类型（精确匹配）。
    pub config_type: Option<String>,
    /// 配置分类（精确匹配）。
    pub category: Option<String>,
    /// 状态过滤（1 启用 / 0 禁用）。
    pub status: Option<i16>,
}

/// 系统参数新增 / 编辑请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ConfigSaveReq {
    /// 配置键。
    #[validate(length(min = 1, max = 100, message = "配置键长度需为 1-100 个字符"))]
    pub config_key: String,
    /// 配置值。
    pub config_value: Option<String>,
    /// 值类型。
    #[validate(length(min = 1, max = 20, message = "值类型长度需为 1-20 个字符"))]
    pub config_type: String,
    /// 配置说明。
    #[validate(length(max = 255, message = "配置说明长度不能超过 255 个字符"))]
    pub description: Option<String>,
    /// 配置分类。
    #[validate(length(max = 50, message = "配置分类长度不能超过 50 个字符"))]
    pub category: Option<String>,
    /// 排序值，缺省 0。
    pub sort_order: Option<i32>,
    /// 配置级别，缺省 2（业务级）。
    pub config_level: Option<i32>,
    /// 是否允许修改，缺省允许。
    pub is_modifiable: Option<bool>,
    /// 是否加密存储，缺省否。
    pub is_encrypted: Option<bool>,
    /// 是否需重启生效，缺省否。
    pub is_restart: Option<bool>,
    /// 状态（1 启用 / 0 禁用），缺省启用。
    pub status: Option<i16>,
    /// 引用值（其他配置键）。
    #[validate(length(max = 500, message = "引用值长度不能超过 500 个字符"))]
    pub ref_value: Option<String>,
    /// 可选值列表。
    #[validate(length(max = 1000, message = "可选值列表长度不能超过 1000 个字符"))]
    pub select_value: Option<String>,
}
