//! 文件元数据的强类型模型与数据访问。
//!
//! 对应迁移 `V0010__create_sys_file`。v1 仅使用本地存储单后端，因此去掉了旧实现
//! 的存储桶字段；上传时间由 `created_at` 承担。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（秒传查询、分页）集中在**本文件**；仅调用框架快捷方法
//! （`find_or_none` / `insert` / `delete` 等）的简单读写由 `handler.rs` 直接执行。

use alun::prelude::*;

use crate::shared::constants::EnableStatus;
use crate::utils;

/// `sys_file` 表对应的强类型模型（文件元数据）。
///
/// # 唯一性与软删
///
/// `file_key` 的部分唯一索引带 `WHERE deleted_at IS NULL`；`file_hash` 用于
/// 秒传（同一 SHA-256 直接复用已存对象）。
///
/// # 大小类型
///
/// `file_size` 对应 `BIGINT NOT NULL`，用 `i64` 承载。
#[derive(Debug, Clone, Model)]
#[table("sys_file")]
pub struct SysFileModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 存储文件名（唯一标识）。
    pub file_name: String,
    /// 原始文件名（用户上传时的名称）。
    pub original_name: Option<String>,
    /// 对象键，软删表上的部分唯一列。
    pub file_key: String,
    /// 文件大小（字节）。
    pub file_size: i64,
    /// 文件 SHA-256（用于秒传与完整性校验）。
    pub file_hash: Option<String>,
    /// MIME 类型。
    pub mime_type: Option<String>,
    /// 扩展名。
    pub extension: Option<String>,
    /// 存储路径。
    pub storage_path: Option<String>,
    /// 分类（`avatar` / `attachment` / `document`）。
    pub category: Option<String>,
    /// 文件状态（1 正常 / 0 已删除）。
    pub status: EnableStatus,
    /// 上传人用户 ID。
    pub uploaded_by: Option<uuid::Uuid>,
    /// 创建时间（即上传时间），由 `TimestampHook` 维护。
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间，由 `TimestampHook` 维护。
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 创建人 ID。
    pub create_by: Option<uuid::Uuid>,
    /// 更新人 ID。
    pub update_by: Option<uuid::Uuid>,
    /// 软删标记（`NULL` = 未删除）。
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ==================== 查询条件与访问函数 ====================

/// 文件分页查询条件。
///
/// `original_name` 保持原始关键词，由本层统一转换为 `LIKE` 模式；`category` /
/// `mime_type` 由处理器去空白后传入，`None` 表示不过滤。
pub(crate) struct FileFilter {
    /// 原始文件名（模糊匹配）。
    pub original_name: Option<String>,
    /// 文件分类（精确匹配）。
    pub category: Option<String>,
    /// MIME 类型（精确匹配）。
    pub mime_type: Option<String>,
}

/// 按内容哈希查询可复用的已存文件（秒传）。
///
/// 命中条件：`file_hash` 相同、未软删、状态为启用。
///
/// # 参数
///
/// - `file_hash`：内容 SHA-256 十六进制摘要。
///
/// # 返回值
///
/// 命中返回已存文件模型，未命中返回 `None`。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn find_reusable_file(
    file_hash: &str,
) -> std::result::Result<Option<SysFileModel>, DbError> {
    let rows = db()
        .find_with_params(
            "SELECT * FROM sys_file WHERE file_hash = ? AND deleted_at IS NULL AND status = 1 LIMIT 1",
            vec![Value::from(file_hash)],
        )
        .await?;
    match rows.into_iter().next() {
        Some(row) => Ok(Some(SysFileModel::from_row(row)?)),
        None => Ok(None),
    }
}

/// 分页查询文件记录。
///
/// # 参数
///
/// - `filter`：过滤条件（原始文件名模糊、分类 / MIME 精确）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `created_at` 倒序的文件分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_files(
    filter: &FileFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysFileModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_file WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.original_name.as_deref()) {
        sql.push_str(" AND original_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(category) = filter.category.as_deref() {
        sql.push_str(" AND category = ?");
        params.push(Value::from(category));
    }
    if let Some(mime_type) = filter.mime_type.as_deref() {
        sql.push_str(" AND mime_type = ?");
        params.push(Value::from(mime_type));
    }
    sql.push_str(" ORDER BY created_at DESC");

    let Page {
        list,
        total,
        page,
        size,
        pages,
    } = db()
        .paginate_with_params(&sql, params, page, page_size)
        .await?;
    let list = list
        .into_iter()
        .map(SysFileModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}
