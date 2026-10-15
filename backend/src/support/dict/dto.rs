//! 数据字典的请求与响应 DTO。
//!
//! 字段命名与前端契约 `frontend/src/api/support/dict-api.ts` 严格对齐：可空列在响应中统一
//! 归一为非空字符串（`unwrap_or_default()`），时间字段按 ISO 8601 下发。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::support::dict::model::{SysDictItemModel, SysDictTypeModel};

/// 字典类型响应。
#[derive(Debug, Clone, Serialize)]
pub struct DictTypeRes {
    /// 字典类型 id。
    pub id: String,
    /// 字典编码。
    pub dict_code: String,
    /// 字典名称。
    pub dict_name: String,
    /// 描述。
    pub description: String,
    /// 状态（1 启用 / 0 禁用）。
    pub status: i16,
    /// 排序值。
    pub sort_order: i32,
    /// 备注。
    pub remark: String,
    /// 创建时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&SysDictTypeModel> for DictTypeRes {
    /// 由字典类型模型构造响应。
    ///
    /// # 参数
    ///
    /// - `m`：字典类型模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的字典类型响应对象。
    fn from(m: &SysDictTypeModel) -> Self {
        Self {
            id: m.id.to_string(),
            dict_code: m.dict_code.clone(),
            dict_name: m.dict_name.clone(),
            description: m.description.clone().unwrap_or_default(),
            status: m.status.as_i16(),
            sort_order: m.sort_order,
            remark: m.remark.clone().unwrap_or_default(),
            created_at: m.created_at,
        }
    }
}

/// 字典项响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictItemRes {
    /// 字典项 id。
    pub id: String,
    /// 所属字典类型 id。
    pub dict_type_id: String,
    /// 字典值。
    pub item_value: String,
    /// 字典标签。
    pub item_label: String,
    /// 描述。
    pub description: String,
    /// 排序值。
    pub sort_order: i32,
    /// 是否默认项。
    pub is_default: bool,
    /// 状态（1 启用 / 0 禁用）。
    pub status: i16,
    /// 创建时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&SysDictItemModel> for DictItemRes {
    /// 由字典项模型构造响应。
    ///
    /// # 参数
    ///
    /// - `m`：字典项模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的字典项响应对象。
    fn from(m: &SysDictItemModel) -> Self {
        Self {
            id: m.id.to_string(),
            dict_type_id: m.dict_type_id.to_string(),
            item_value: m.item_value.clone(),
            item_label: m.item_label.clone(),
            description: m.description.clone().unwrap_or_default(),
            sort_order: m.sort_order,
            is_default: m.is_default,
            status: m.status.as_i16(),
            created_at: m.created_at,
        }
    }
}

/// 字典类型分页查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DictTypePageQuery {
    /// 页码，从 1 开始（缺省为 1）。
    pub page: Option<u64>,
    /// 每页条数（缺省为框架默认值，超出上限会被规整）。
    pub page_size: Option<u64>,
    /// 字典编码（模糊匹配）。
    pub dict_code: Option<String>,
    /// 字典名称（模糊匹配）。
    pub dict_name: Option<String>,
    /// 状态过滤（1 启用 / 0 禁用）。
    pub status: Option<i16>,
}

/// 字典类型新增 / 编辑请求。
#[derive(Debug, Deserialize, Validate)]
pub struct DictTypeSaveReq {
    /// 字典编码。
    #[validate(length(min = 1, max = 50, message = "字典编码长度需为 1-50 个字符"))]
    pub dict_code: String,
    /// 字典名称。
    #[validate(length(min = 1, max = 100, message = "字典名称长度需为 1-100 个字符"))]
    pub dict_name: String,
    /// 描述。
    #[validate(length(max = 255, message = "描述长度不能超过 255 个字符"))]
    pub description: Option<String>,
    /// 状态（1 启用 / 0 禁用），缺省启用。
    pub status: Option<i16>,
    /// 排序值，缺省 0。
    pub sort_order: Option<i32>,
    /// 备注（数据库为 TEXT，不做长度约束）。
    pub remark: Option<String>,
}

/// 字典项分页查询参数。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DictItemPageQuery {
    /// 页码，从 1 开始（缺省为 1）。
    pub page: Option<u64>,
    /// 每页条数（缺省为框架默认值，超出上限会被规整）。
    pub page_size: Option<u64>,
    /// 所属字典类型 id。
    pub dict_type_id: Option<String>,
    /// 字典编码（按所属字典类型的编码精确过滤）。
    pub dict_code: Option<String>,
    /// 字典标签（模糊匹配）。
    pub item_label: Option<String>,
    /// 状态过滤（1 启用 / 0 禁用）。
    pub status: Option<i16>,
}

/// 字典项新增 / 编辑请求。
#[derive(Debug, Deserialize, Validate)]
pub struct DictItemSaveReq {
    /// 所属字典类型 id。
    #[validate(length(min = 1, max = 64, message = "字典类型 id 不能为空"))]
    pub dict_type_id: String,
    /// 字典值。
    #[validate(length(min = 1, max = 100, message = "字典值长度需为 1-100 个字符"))]
    pub item_value: String,
    /// 字典标签。
    #[validate(length(min = 1, max = 200, message = "字典标签长度需为 1-200 个字符"))]
    pub item_label: String,
    /// 描述。
    #[validate(length(max = 255, message = "描述长度不能超过 255 个字符"))]
    pub description: Option<String>,
    /// 排序值，缺省 0。
    pub sort_order: Option<i32>,
    /// 是否默认项，缺省否。
    pub is_default: Option<bool>,
    /// 状态（1 启用 / 0 禁用），缺省启用。
    pub status: Option<i16>,
}
