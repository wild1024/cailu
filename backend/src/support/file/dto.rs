//! 文件管理的请求与响应 DTO。
//!
//! 字段名与前端契约 `frontend/src/api/support/file-api.ts` 严格对齐：响应中的 `url` 由后端拼装，
//! 取值恒为 `/api/support/file/download/{id}`，供前端做下载 / 预览。

use alun::prelude::*;
use serde::{Deserialize, Serialize};

use crate::support::file::model::SysFileModel;

/// 文件记录响应。
///
/// 与前端 `FileRes` 定义一一对应；可选列（原始文件名、MIME、扩展名、分类、
/// 上传人）统一以空串兜底，保证前端拿到稳定的字符串契约。
#[derive(Debug, Clone, Serialize)]
pub struct FileRes {
    /// 文件 ID（UUID 字符串）。
    pub id: String,
    /// 存储文件名（与 `file_key` 相同）。
    pub file_name: String,
    /// 原始文件名（用户上传时的名称）。
    pub original_name: String,
    /// 存储 key（落盘文件名）。
    pub file_key: String,
    /// 文件大小（字节）。
    pub file_size: i64,
    /// MIME 类型。
    pub mime_type: String,
    /// 扩展名（小写，无扩展名时为空串）。
    pub extension: String,
    /// 文件分类（如 `avatar` / `attachment` / `document`）。
    pub category: String,
    /// 状态（1 启用 / 0 禁用）。
    pub status: i16,
    /// 上传人用户 ID。
    pub uploaded_by: String,
    /// 上传时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 访问地址（下载 / 预览）。
    pub url: String,
}

impl From<&SysFileModel> for FileRes {
    /// 由文件模型构造响应对象。
    ///
    /// # 参数
    ///
    /// - `m`：文件元数据模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的 [`FileRes`]，其中 `url` 指向本记录的下载接口。
    fn from(m: &SysFileModel) -> Self {
        Self {
            id: m.id.to_string(),
            file_name: m.file_name.clone(),
            original_name: m.original_name.clone().unwrap_or_default(),
            file_key: m.file_key.clone(),
            file_size: m.file_size,
            mime_type: m.mime_type.clone().unwrap_or_default(),
            extension: m.extension.clone().unwrap_or_default(),
            category: m.category.clone().unwrap_or_default(),
            status: m.status.as_i16(),
            uploaded_by: m.uploaded_by.map(|v| v.to_string()).unwrap_or_default(),
            created_at: m.created_at,
            url: format!("/api/support/file/download/{}", m.id),
        }
    }
}

/// 文件分页查询参数。
///
/// `page` / `page_size` 缺省时由 [`crate::utils::normalize_page`] 规整到默认值；
/// 其余字段均为可选过滤条件。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FilePageQuery {
    /// 页码（从 1 开始）。
    pub page: Option<u64>,
    /// 每页条数。
    pub page_size: Option<u64>,
    /// 原始文件名（模糊匹配）。
    pub original_name: Option<String>,
    /// 文件分类（精确匹配）。
    pub category: Option<String>,
    /// MIME 类型（精确匹配）。
    pub mime_type: Option<String>,
}

/// 文件上传的查询参数。
///
/// 上传走 `multipart/form-data`，分类信息无法放在表单字段中，故经查询参数传递。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileUploadQuery {
    /// 文件分类；缺省为 `attachment`。
    pub category: Option<String>,
}

/// 秒传预检的查询参数。
#[derive(Debug, Clone, Deserialize)]
pub struct FileCheckDuplicateQuery {
    /// 待检测文件内容的 SHA-256 十六进制摘要。
    pub file_hash: String,
}

/// 秒传预检响应。
#[derive(Debug, Clone, Serialize)]
pub struct FileDuplicateRes {
    /// 是否已存在内容相同的可复用文件。
    pub duplicated: bool,
    /// 命中时返回已存文件记录（未命中为 `null`），前端可据此跳过上传。
    pub file: Option<FileRes>,
}
