//! 文件管理处理器：上传（含内容哈希秒传）、分页、详情、删除、下载。
//!
//! 处理器承担**参数校验、磁盘 I/O、业务编排与响应组装**，并直接调用框架快捷方法
//! （`find_or_none` / `insert` / `delete` 等）；需要编写 SQL 的查询（分页、按内容
//! 哈希复用）集中在 [`crate::support::file::model`]。上传 / 下载目录取自框架全局
//! 访问器 `alun::upload_path()`（`[upload].path`）；上传大小上限取自
//! `cfg().upload.max_size_mb`（`[upload].max_size_mb`）。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::{ConnectInfo, Multipart};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

use std::net::SocketAddr;

use crate::shared::constants::EnableStatus;
use crate::shared::error_code;
use crate::support::file::dto::{
    FileCheckDuplicateQuery, FileDuplicateRes, FilePageQuery, FileRes, FileUploadQuery,
};
use crate::support::file::model::{self, SysFileModel};
use crate::support::log;
use crate::utils;

/// 允许上传的文件扩展名白名单（小写、不含点）。
///
/// 覆盖常见图片 / 文档 / 压缩包 / 音视频类型；不在白名单或无扩展名的文件拒绝上传，
/// 防止可执行脚本等危险类型落盘。
const ALLOWED_UPLOAD_EXTENSIONS: [&str; 31] = [
    // 图片
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", // 文档
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "csv", "md", "json", "xml",
    // 压缩包
    "zip", "rar", "7z", "gz", "tar", // 音视频
    "mp3", "wav", "mp4", "avi", "mov", "mkv",
];

// ==================== 接口 ====================

/// 上传文件（multipart 字段名固定为 `file`，`?category=` 可选）。
///
/// # 参数
///
/// - `query`：查询参数（`category`）。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `multipart`：多部分表单（须为最后一个提取器）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`FILE_UPLOAD`，`after_data` 记录原始名 / 大小 / MIME）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回文件记录响应；命中相同内容哈希时复用已存记录（秒传）。
///
/// # 异常情况
///
/// 表单非法返回 400，超限返回 422，写盘 / 入库失败返回 500。
#[alun::permission(
    path = "/support/file/upload",
    method = "POST",
    permission = "support:file:upload"
)]
#[alun::post("/support/file/upload")]
pub async fn upload(
    Query(query): Query<FileUploadQuery>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    multipart: Multipart,
) -> ResResult<FileRes> {
    let mut files = read_upload_fields(multipart).await?;
    let (original_name, mime_type, bytes) = files.swap_remove(0);
    let stored = store_file_bytes(&query, &claims, original_name, mime_type, bytes).await?;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "FILE_UPLOAD",
            "File",
            stored.id.clone(),
            None,
            Some(serde_json::json!({
                "original_name": stored.original_name,
                "file_size": stored.file_size,
                "mime_type": stored.mime_type,
            })),
        ),
    );
    Ok(Res::ok(stored))
}

/// 批量上传文件（multipart 中可携带多个同名 `file` 字段，`?category=` 可选）。
///
/// 逐个走与单文件上传完全一致的大小 / 类型 / 秒传 / 落盘 / 入库逻辑，按表单顺序
/// 返回每个文件的记录（含秒传命中复用的记录）。
///
/// # 参数
///
/// - `query`：查询参数（`category`）。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `multipart`：多部分表单（须为最后一个提取器）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`FILE_UPLOAD_BATCH`，`target_id` 为本批文件 ID 逗号拼接，
/// `after_data` 记录本批数量与原始文件名列表）；日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回本批每个文件的记录列表。
///
/// # 异常情况
///
/// 表单非法返回 400，任一文件超限 / 类型不允许返回 422 / 400，写盘 / 入库失败返回 500。
#[alun::permission(
    path = "/support/file/upload/batch",
    method = "POST",
    permission = "support:file:upload"
)]
#[alun::post("/support/file/upload/batch")]
pub async fn upload_batch(
    Query(query): Query<FileUploadQuery>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    multipart: Multipart,
) -> ResResult<Vec<FileRes>> {
    let files = read_upload_fields(multipart).await?;
    let mut result = Vec::with_capacity(files.len());
    for (original_name, mime_type, bytes) in files {
        result.push(store_file_bytes(&query, &claims, original_name, mime_type, bytes).await?);
    }
    let target_id = result
        .iter()
        .map(|file| file.id.clone())
        .collect::<Vec<_>>()
        .join(",");
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "FILE_UPLOAD_BATCH",
            "File",
            target_id,
            None,
            Some(serde_json::json!({
                "count": result.len(),
                "file_names": result.iter().map(|file| file.original_name.clone()).collect::<Vec<_>>(),
            })),
        ),
    );
    Ok(Res::ok(result))
}

/// 秒传预检：按内容哈希判断是否已存在相同文件。
///
/// 前端在真正上传前先计算文件 SHA-256 调用本接口；若 `duplicated = true`，可直接
/// 取用返回的 `file` 记录而无需再次传输文件内容。
///
/// # 异常情况
///
/// `file_hash` 为空返回 400，数据库异常返回 500。
#[alun::permission(
    path = "/support/file/check-duplicate",
    method = "GET",
    permission = "support:file:upload"
)]
#[alun::get("/support/file/check-duplicate")]
pub async fn check_duplicate(
    Query(query): Query<FileCheckDuplicateQuery>,
) -> ResResult<FileDuplicateRes> {
    let file_hash = query.file_hash.trim();
    if file_hash.is_empty() {
        return Err(ApiError::new(
            400,
            error_code::PARAM_INVALID,
            "file_hash 不能为空",
        ));
    }
    let existing = model::find_reusable_file(file_hash)
        .await
        .map_err(|e| utils::db_error(e, "秒传预检"))?;
    Ok(Res::ok(FileDuplicateRes {
        duplicated: existing.is_some(),
        file: existing.as_ref().map(FileRes::from),
    }))
}

/// 分页查询文件（`original_name` / `category` / `mime_type` 均可选）。
///
/// # 参数
///
/// - `query`：分页与过滤条件。
///
/// # 返回值
///
/// 成功返回分页文件列表（按上传时间倒序）。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/support/file/page",
    method = "GET",
    permission = "support:file:list"
)]
#[alun::get("/support/file/page")]
pub async fn page(Query(query): Query<FilePageQuery>) -> ResResult<PageData<Vec<FileRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    let filter = model::FileFilter {
        original_name: query.original_name,
        // 精确匹配前统一去空白，空白值视为未提供
        category: utils::trimmed(query.category.as_deref()).map(str::to_string),
        mime_type: utils::trimmed(query.mime_type.as_deref()).map(str::to_string),
    };

    let result = model::page_files(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询文件"))?;
    let list = result.list.iter().map(FileRes::from).collect::<Vec<_>>();

    Ok(Res::page(list, result.total, result.page, result.size))
}

/// 文件详情。
///
/// # 参数
///
/// - `id`：文件 ID。
///
/// # 返回值
///
/// 成功返回文件记录。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：文件不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/file/{id}",
    method = "GET",
    permission = "support:file:list"
)]
#[alun::get("/support/file/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<FileRes> {
    let id = utils::require_uuid(&id, "id")?;
    let file = SysFileModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询文件详情"))?
        .ok_or_else(|| ApiError::new(404, error_code::FILE_NOT_FOUND, "文件不存在"))?;
    Ok(Res::ok(FileRes::from(&file)))
}

/// 删除文件（软删元数据，并尽力删除磁盘文件）。
///
/// # 参数
///
/// - `id`：文件 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`FILE_DELETE`，`before_data` 记录被删文件的原始名 / 存储键）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。磁盘文件删除失败只告警，不影响接口成功。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：文件不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/file/{id}",
    method = "DELETE",
    permission = "support:file:delete"
)]
#[alun::delete("/support/file/{id}")]
pub async fn delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let file = SysFileModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待删除文件"))?
        .ok_or_else(|| ApiError::new(404, error_code::FILE_NOT_FOUND, "文件不存在"))?;

    SysFileModel::of(id)
        .delete(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "删除文件记录"))?;

    // 尽力删除磁盘文件，失败仅告警
    let path = format!("{}/{}", alun::upload_path(), file.file_key);
    if let Err(e) = std::fs::remove_file(&path) {
        tracing::warn!(path = %path, error = %e, "删除磁盘文件失败，仅软删元数据");
    }
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "FILE_DELETE",
            "File",
            id.to_string(),
            Some(serde_json::json!({
                "original_name": file.original_name,
                "file_key": file.file_key,
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

/// 下载文件（返回二进制响应，非统一响应体）。
///
/// # 参数
///
/// - `id`：文件 ID。
///
/// # 返回值
///
/// `Content-Type` 取记录 MIME（缺省 `application/octet-stream`），
/// `Content-Disposition` 为 `attachment`（中文名走 RFC 5987 编码），
/// body 为文件字节。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：文件不存在或磁盘文件已丢失；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/file/download/{id}",
    method = "GET",
    permission = "support:file:list"
)]
#[alun::get("/support/file/download/{id}")]
pub async fn download(Path(id): Path<String>) -> Response {
    match download_response(&id).await {
        Ok(resp) => resp,
        Err(e) => e.into_response(),
    }
}

// ==================== 公共辅助 ====================

/// 解析原始文件名的扩展名。
///
/// # 参数
///
/// - `name`：原始文件名。
///
/// # 返回值
///
/// 最后一个 `.` 之后的部分（转小写）；无 `.` 或 `.` 后为空时返回空串。
fn extension_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// 计算内容 SHA-256 十六进制小写摘要。
///
/// # 参数
///
/// - `bytes`：文件字节。
///
/// # 返回值
///
/// 64 位小写十六进制字符串，用于秒传命中和完整性校验。
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// 从 multipart 表单中读取所有字段名为 `file` 的文件（支持批量上传）。
///
/// # 参数
///
/// - `multipart`：axum 多部分表单提取器。
///
/// # 返回值
///
/// 按表单顺序返回 `(原始文件名, content_type, 字节)` 列表；原始文件名缺失时为空串，
/// content_type 缺失时为 `application/octet-stream`。
///
/// # 异常情况
///
/// - 400：表单解析失败 / 文件内容读取失败 / 不存在任何 `file` 字段。
async fn read_upload_fields(
    mut multipart: Multipart,
) -> std::result::Result<Vec<(String, String, Vec<u8>)>, ApiError> {
    let mut files = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::new(400, error_code::PAYLOAD_INVALID, "上传表单解析失败"))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let original_name = field.file_name().unwrap_or("").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = field
            .bytes()
            .await
            .map_err(|_| ApiError::new(400, error_code::PAYLOAD_INVALID, "上传文件内容读取失败"))?;
        files.push((original_name, content_type, bytes.to_vec()));
    }
    if files.is_empty() {
        return Err(ApiError::new(
            400,
            error_code::PARAM_INVALID,
            "缺少文件字段 file",
        ));
    }
    Ok(files)
}

/// 校验扩展名是否在上传白名单内。
///
/// # 参数
///
/// - `extension`：已转小写、不含点的扩展名；空串视为无扩展名。
///
/// # 返回值
///
/// 合法返回 `Ok(())`。
///
/// # 异常情况
///
/// 无扩展名或不在白名单返回 400（`FILE_TYPE_NOT_ALLOWED`）。
fn ensure_extension_allowed(extension: &str) -> std::result::Result<(), ApiError> {
    if extension.is_empty() || !ALLOWED_UPLOAD_EXTENSIONS.contains(&extension) {
        return Err(ApiError::new(
            400,
            error_code::FILE_TYPE_NOT_ALLOWED,
            format!("不允许上传该类型文件：{extension}"),
        ));
    }
    Ok(())
}

/// 处理单个文件的落盘与入库（含大小 / 类型校验与内容哈希秒传）。
///
/// # 参数
///
/// - `query`：上传查询参数（`category`，缺省 `attachment`）。
/// - `claims`：当前用户声明，用于写入上传人。
/// - `original_name`：原始文件名。
/// - `mime_type`：MIME 类型。
/// - `bytes`：文件字节。
///
/// # 返回值
///
/// 文件记录响应（首传为新建记录，秒传为复用记录）。
///
/// # 异常情况
///
/// - 400：文件类型不在白名单；
/// - 422：文件大小超过 `[upload].max_size_mb`；
/// - 500：创建目录 / 写盘 / 数据库写入失败。
async fn store_file_bytes(
    query: &FileUploadQuery,
    claims: &TokenClaims,
    original_name: String,
    mime_type: String,
    bytes: Vec<u8>,
) -> std::result::Result<FileRes, ApiError> {
    // 大小校验：上限取 [upload].max_size_mb（MB）
    let max_mb = cfg().upload.max_size_mb;
    let max_bytes = max_mb.saturating_mul(1024 * 1024);
    if bytes.len() as u64 > max_bytes {
        return Err(ApiError::new(
            422,
            error_code::FILE_SIZE_EXCEEDED,
            format!("文件大小超过上限 {max_mb}MB"),
        ));
    }

    // 内容哈希：命中则直接复用已存记录，不再落盘、不再插新行（秒传）
    let file_hash = sha256_hex(&bytes);
    let existing = model::find_reusable_file(&file_hash)
        .await
        .map_err(|e| utils::db_error(e, "查询可复用文件"))?;
    if let Some(existing) = existing {
        tracing::info!(
            file_hash = %file_hash,
            file_key = %existing.file_key,
            "命中文件秒传，复用已存记录"
        );
        return Ok(FileRes::from(&existing));
    }

    let id = uuid::Uuid::now_v7();
    let extension = extension_of(&original_name);
    // 新文件落盘前校验扩展名白名单（秒传复用的历史记录已合规，无需再拦）
    ensure_extension_allowed(&extension)?;
    // 有扩展名时补前导点，等价于「id + .ext」
    let suffix = if extension.is_empty() {
        String::new()
    } else {
        format!(".{extension}")
    };
    let file_key = format!("{id}{suffix}");
    let file_name = file_key.clone();
    let dir = alun::upload_path();
    let path = format!("{dir}/{file_key}");

    if let Err(e) = std::fs::create_dir_all(dir) {
        tracing::error!(dir = %dir, error = %e, "创建上传目录失败");
        return Err(ApiError::new(
            500,
            error_code::INTERNAL_ERROR,
            "服务器内部错误",
        ));
    }
    if let Err(e) = std::fs::write(&path, &bytes) {
        tracing::error!(path = %path, error = %e, "写入上传文件失败");
        return Err(ApiError::new(
            500,
            error_code::INTERNAL_ERROR,
            "服务器内部错误",
        ));
    }

    let category = utils::trimmed(query.category.as_deref())
        .unwrap_or("attachment")
        .to_string();
    let operator = utils::current_user_uuid(claims);
    let now = chrono::Utc::now();
    let mut file_record = SysFileModel {
        id,
        file_name,
        original_name: Some(original_name),
        file_key,
        file_size: bytes.len() as i64,
        file_hash: Some(file_hash),
        mime_type: Some(mime_type),
        extension: Some(extension),
        storage_path: Some(path),
        category: Some(category),
        status: EnableStatus::enabled,
        uploaded_by: operator,
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    };
    file_record
        .insert(db())
        .await
        .map_err(|e| utils::db_error(e, "写入文件元数据"))?;
    Ok(FileRes::from(&file_record))
}

/// 把文件名降级为 ASCII 安全的回退名（供 `filename="..."` 使用）。
///
/// # 参数
///
/// - `value`：原始文件名。
///
/// # 返回值
///
/// 非 ASCII、控制字符、引号与反斜杠替换为 `_` 后的字符串。
fn ascii_fallback(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii() && c != '"' && c != '\\' && !c.is_control() {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// 对文件名做 RFC 5987 百分号编码（attr-char 之外一律编码）。
///
/// # 参数
///
/// - `value`：原始文件名。
///
/// # 返回值
///
/// 供 `filename*=UTF-8''<encoded>` 使用的编码串。
fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for b in value.as_bytes() {
        let byte = *b;
        let keep = byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'.' | b'_' | b'~' | b'!' | b'#' | b'$' | b'&' | b'+' | b'^' | b'`' | b'|'
            );
        if keep {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// 构造 `Content-Disposition` 头（同时给出 ASCII 回退名与 RFC 5987 编码名）。
///
/// # 参数
///
/// - `original`：原始文件名。
///
/// # 返回值
///
/// 合法的 `HeaderValue`；非法字符兜底为仅 `attachment`。
fn build_content_disposition(original: &str) -> HeaderValue {
    let ascii = ascii_fallback(original);
    let encoded = percent_encode(original);
    let value = format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}");
    HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

/// 读取文件记录与磁盘内容并组装二进制响应。
///
/// # 参数
///
/// - `raw_id`：路径中的文件 ID 字符串。
///
/// # 返回值
///
/// `Content-Type` 取记录 MIME（缺省 `application/octet-stream`）、
/// `Content-Disposition` 为附件下载的裸响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：记录不存在或磁盘文件已丢失；
/// - 500：数据库异常。
async fn download_response(raw_id: &str) -> std::result::Result<Response, ApiError> {
    let id = utils::require_uuid(raw_id, "id")?;
    let file = SysFileModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询下载文件"))?
        .ok_or_else(|| ApiError::new(404, error_code::FILE_NOT_FOUND, "文件不存在"))?;

    let path = format!("{}/{}", alun::upload_path(), file.file_key);
    let bytes = std::fs::read(&path)
        .map_err(|_| ApiError::new(404, error_code::FILE_NOT_FOUND, "文件已丢失"))?;

    let mime = file
        .mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let content_type = HeaderValue::from_str(&mime)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let original = file
        .original_name
        .clone()
        .unwrap_or_else(|| file.file_name.clone());
    let disposition = build_content_disposition(&original);

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response())
}
