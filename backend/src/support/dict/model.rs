//! 数据字典的强类型模型与数据访问。
//!
//! 对应迁移 `V0007__create_sys_dict_type` 与 `V0008__create_sys_dict_item`。
//! 字典类型（`sys_dict_type`）与字典项（`sys_dict_item`）是一对多关系，
//! 由 `sys_dict_item.dict_type_id` 建立外键。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（分页查询、唯一性统计、按编码查询、启用项列表）与缓存逻辑
//! 集中在**本文件**；仅调用框架快捷方法（`find_or_none` / `insert` / `update` /
//! `delete` 等）的简单读写由 `handler.rs` 直接执行。字典项列表按 `dict_code`
//! 过滤时 JOIN `sys_dict_type`（编码承载在类型表上，字典项表只存 `dict_type_id`）。
//!
//! # 缓存
//!
//! 「按 `dict_code` 查询启用字典项」是前端下拉复用的高频读接口，结果按
//! `dict:items:{dict_code}` 缓存：启动时预热内置编码（[`warm_dict_cache`]），
//! 读路径缓存优先并回填，字典类型 / 字典项写操作主动失效对应键。

use alun::prelude::*;

use crate::shared::constants;
use crate::shared::constants::EnableStatus;
use crate::support::dict::dto::DictItemRes;
use crate::utils;

/// `sys_dict_type` 表对应的强类型模型（字典类型）。
///
/// # 唯一性与软删
///
/// `dict_code` 的部分唯一索引带 `WHERE deleted_at IS NULL`；基础字典编码见
/// [`crate::shared::constants`]（`user_status` / `menu_type` / `config_type`
/// / `job_status`）。
#[derive(Debug, Clone, Model)]
#[table("sys_dict_type")]
pub struct SysDictTypeModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 字典编码，软删表上的部分唯一列。
    pub dict_code: String,
    /// 字典名称。
    pub dict_name: String,
    /// 描述。
    pub description: Option<String>,
    /// 字典状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 排序号，升序展示。
    pub sort_order: i32,
    /// 备注。
    pub remark: Option<String>,
    /// 创建时间，由 `TimestampHook` 维护。
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

/// `sys_dict_item` 表对应的强类型模型（字典项）。
///
/// # 唯一性与软删
///
/// 部分唯一索引 `(dict_type_id, item_value) WHERE deleted_at IS NULL`：同一字典
/// 类型下 `item_value` 唯一，删除后可重建同值项。
#[derive(Debug, Clone, Model)]
#[table("sys_dict_item")]
pub struct SysDictItemModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 所属字典类型 ID，引用 `sys_dict_type(id)`。
    pub dict_type_id: uuid::Uuid,
    /// 字典项值（落库值，部分唯一索引的组成部分）。
    pub item_value: String,
    /// 字典项标签（展示文案）。
    pub item_label: String,
    /// 描述。
    pub description: Option<String>,
    /// 排序号，升序展示。
    pub sort_order: i32,
    /// 是否为该字典的默认项。
    pub is_default: bool,
    /// 字典项状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 创建时间，由 `TimestampHook` 维护。
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

// ==================== 查询条件 ====================

/// 字典类型分页查询条件。
///
/// 由处理器完成状态值校验后传入；模糊匹配字段保持原始关键词，由本层统一转换为
/// `LIKE` 模式，避免查询细节泄漏到处理器。
pub(crate) struct DictTypeFilter {
    /// 字典编码（模糊匹配关键词）。
    pub dict_code: Option<String>,
    /// 字典名称（模糊匹配关键词）。
    pub dict_name: Option<String>,
    /// 状态（调用方已校验为合法枚举值）。
    pub status: Option<EnableStatus>,
}

/// 字典项分页查询条件。
///
/// `dict_type_id` 与 `dict_code` 已由处理器完成解析与去空白；`item_label` 保持原始
/// 关键词交由本层转换为 `LIKE` 模式。
pub(crate) struct DictItemFilter {
    /// 所属字典类型 ID。
    pub dict_type_id: Option<uuid::Uuid>,
    /// 所属字典类型的编码（精确匹配，依赖 JOIN `sys_dict_type`）。
    pub dict_code: Option<String>,
    /// 字典标签（模糊匹配关键词）。
    pub item_label: Option<String>,
    /// 状态（调用方已校验为合法枚举值）。
    pub status: Option<EnableStatus>,
}

// ==================== 字典类型访问 ====================

/// 分页查询字典类型。
///
/// # 参数
///
/// - `filter`：过滤条件（编码 / 名称模糊，状态精确）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `sort_order` 升序的字典类型分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_dict_types(
    filter: &DictTypeFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysDictTypeModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_dict_type WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.dict_code.as_deref()) {
        sql.push_str(" AND dict_code LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(pattern) = utils::like_pattern(filter.dict_name.as_deref()) {
        sql.push_str(" AND dict_name LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    sql.push_str(" ORDER BY sort_order ASC, created_at ASC");

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
        .map(SysDictTypeModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 按字典编码查询字典类型（仅未删除记录）。
///
/// # 参数
///
/// - `dict_code`：字典编码。
///
/// # 返回值
///
/// 命中返回字典类型模型，未命中返回 `None`。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn find_dict_type_by_code(
    dict_code: &str,
) -> std::result::Result<Option<SysDictTypeModel>, DbError> {
    SysDictTypeModel::find_first_by(db(), "dict_code = ? AND deleted_at IS NULL", (dict_code,))
        .await
}

/// 统计字典编码在未删除记录中的数量（唯一性校验）。
///
/// # 参数
///
/// - `dict_code`：字典编码。
/// - `exclude_id`：需要排除的主键（更新时排除自身）。
///
/// # 返回值
///
/// 命中条数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_dict_code(
    dict_code: &str,
    exclude_id: Option<&uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude_id {
        Some(id) => {
            SysDictTypeModel::count_by(
                db(),
                "dict_code = ? AND deleted_at IS NULL AND id <> ?",
                (dict_code, id),
            )
            .await
        }
        None => {
            SysDictTypeModel::count_by(db(), "dict_code = ? AND deleted_at IS NULL", (dict_code,))
                .await
        }
    }
}

// ==================== 字典项访问 ====================

/// 分页查询字典项。
///
/// # 参数
///
/// - `filter`：过滤条件（所属类型 ID / 编码精确，标签模糊，状态）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `sort_order` 升序的字典项分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_dict_items(
    filter: &DictItemFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysDictItemModel>, DbError> {
    let mut sql = String::from(
        "SELECT i.* FROM sys_dict_item i \
         INNER JOIN sys_dict_type t ON t.id = i.dict_type_id \
         WHERE i.deleted_at IS NULL AND t.deleted_at IS NULL",
    );
    let mut params: Vec<Value> = Vec::new();
    if let Some(dict_type_id) = filter.dict_type_id {
        sql.push_str(" AND i.dict_type_id = ?");
        params.push(Value::from(&dict_type_id));
    }
    if let Some(code) = filter.dict_code.as_deref() {
        sql.push_str(" AND t.dict_code = ?");
        params.push(Value::from(code));
    }
    if let Some(pattern) = utils::like_pattern(filter.item_label.as_deref()) {
        sql.push_str(" AND i.item_label LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND i.status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    sql.push_str(" ORDER BY i.sort_order ASC, i.created_at ASC");

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
        .map(SysDictItemModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 统计某字典类型下的字典项数量（未删除记录）。
///
/// # 参数
///
/// - `dict_type_id`：字典类型 ID。
///
/// # 返回值
///
/// 命中条数（用于删除类型前的引用校验）。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_items_by_type(
    dict_type_id: &uuid::Uuid,
) -> std::result::Result<i64, DbError> {
    SysDictItemModel::count_by(
        db(),
        "dict_type_id = ? AND deleted_at IS NULL",
        (dict_type_id,),
    )
    .await
}

/// 查询某字典类型下的全部启用字典项（未删除）。
///
/// # 参数
///
/// - `dict_type_id`：字典类型 ID。
///
/// # 返回值
///
/// 按 `sort_order` 升序的启用字典项列表。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn list_enabled_dict_items(
    dict_type_id: &uuid::Uuid,
) -> std::result::Result<Vec<SysDictItemModel>, DbError> {
    let rows = db()
        .find_with_params(
            "SELECT * FROM sys_dict_item \
             WHERE dict_type_id = ? AND status = ? AND deleted_at IS NULL \
             ORDER BY sort_order ASC",
            vec![
                Value::from(dict_type_id),
                TypeMapper::to_value(&constants::STATUS_ENABLED),
            ],
        )
        .await?;
    rows.into_iter()
        .map(SysDictItemModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()
}

/// 统计同一字典类型下某字典值在未删除记录中的数量（唯一性校验）。
///
/// # 参数
///
/// - `dict_type_id`：字典类型 ID。
/// - `item_value`：字典值。
/// - `exclude_id`：需要排除的主键（更新时排除自身）。
///
/// # 返回值
///
/// 命中条数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_dict_item_value(
    dict_type_id: &uuid::Uuid,
    item_value: &str,
    exclude_id: Option<&uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude_id {
        Some(id) => {
            SysDictItemModel::count_by(
                db(),
                "dict_type_id = ? AND item_value = ? AND deleted_at IS NULL AND id <> ?",
                (dict_type_id, item_value, id),
            )
            .await
        }
        None => {
            SysDictItemModel::count_by(
                db(),
                "dict_type_id = ? AND item_value = ? AND deleted_at IS NULL",
                (dict_type_id, item_value),
            )
            .await
        }
    }
}

// ==================== 字典项缓存 ====================

/// 由字典编码拼装缓存键。
///
/// # 参数
///
/// - `dict_code`：字典编码。
///
/// # 返回值
///
/// 形如 `dict:items:{dict_code}` 的缓存键。
fn cache_key(dict_code: &str) -> String {
    format!("{}{}", constants::CACHE_KEY_PREFIX_DICT, dict_code)
}

/// 读取按字典编码缓存的启用字典项列表。
///
/// # 参数
///
/// - `dict_code`：字典编码。
///
/// # 返回值
///
/// 命中返回 `Some(列表)`，键不存在（缓存未预热或已过期）返回 `None`。
///
/// # 异常情况
///
/// 缓存未初始化或读取失败返回错误，由调用方决定是否回退数据库查询。
pub(crate) async fn load_cached_items(
    dict_code: &str,
) -> std::result::Result<Option<Vec<DictItemRes>>, Error> {
    let Some(store) = try_cache() else {
        return Err(Error::Msg("缓存未初始化，无法读取字典缓存".to_string()));
    };
    Cache::get(store, &cache_key(dict_code)).await
}

/// 写入按字典编码缓存的启用字典项列表。
///
/// # 参数
///
/// - `dict_code`：字典编码。
/// - `items`：启用字典项响应列表。
///
/// # 异常情况
///
/// 缓存未初始化或写入失败返回错误；调用方可只记日志（缓存非主数据源）。
pub(crate) async fn cache_items(
    dict_code: &str,
    items: &[DictItemRes],
) -> std::result::Result<(), Error> {
    let Some(store) = try_cache() else {
        return Err(Error::Msg("缓存未初始化，无法写入字典缓存".to_string()));
    };
    Cache::set_ex(
        store,
        &cache_key(dict_code),
        items,
        constants::CACHE_DICT_TTL_SECS,
    )
    .await
}

/// 失效指定字典编码的缓存（写操作后调用，避免读路径返回旧数据）。
///
/// 键不存在不视为错误；缓存未初始化时直接跳过（无缓存即无脏读）。
///
/// # 参数
///
/// - `dict_code`：字典编码。
pub(crate) async fn invalidate_cache(dict_code: &str) {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::del(store, &cache_key(dict_code)).await {
        tracing::warn!(dict_code, error = %e, "失效字典缓存失败");
    }
}

/// 按字典类型 ID 解析其编码后失效缓存（供字典项写操作调用）。
///
/// 字典项表只存 `dict_type_id`，需回查类型表取得编码再删除对应缓存键。
///
/// # 参数
///
/// - `dict_type_id`：字典类型 ID。
pub(crate) async fn invalidate_cache_by_type_id(dict_type_id: &uuid::Uuid) {
    match SysDictTypeModel::find_or_none(db(), dict_type_id).await {
        Ok(Some(dict_type)) => invalidate_cache(&dict_type.dict_code).await,
        Ok(None) => {}
        Err(e) => tracing::warn!(?dict_type_id, error = %e, "解析字典编码以失效缓存失败"),
    }
}

/// 预热内置字典缓存。
///
/// 对 [`constants::WARM_DICT_CODES`] 中的每个编码，读取启用字典项并写入缓存，
/// 使高频下拉接口首次访问即命中。单个编码预热失败只记告警，不影响其他编码。
///
/// # 异常情况
///
/// 本函数不返回错误：预热属启动期优化，失败降级为「命中时再回填」。
pub(crate) async fn warm_dict_cache() {
    // 限并发 2 预热（与 pro 版同款实测组合）：各编码互不依赖，完全串行会把
    // 10×2 次 DB 往返累加进启动；但并发度也不宜拉满——空闲连接不足的并发
    // 请求会**现建连接**，建连成本高于并发收益（pro 实测 10 并发反而更慢）。
    // 失败处理逐项不变（见 `warm_one_dict`）。
    //
    // 用纯 `tokio::join!` 分批而非 `futures_util`：基础版不引入该依赖；
    // `WARM_DICT_CODES` 为 const 数组（元素 `&str` 为 Copy），按值拷入局部
    // 后分块借用，保证 `on_startup` 钩子 future 仍是 `'static`。
    let codes = constants::WARM_DICT_CODES;
    for chunk in codes.chunks(2) {
        if let [a, b] = chunk {
            let _ = tokio::join!(warm_one_dict(a), warm_one_dict(b));
        } else {
            for code in chunk {
                warm_one_dict(code).await;
            }
        }
    }
}

/// 预热单个字典编码：读取启用字典项并写入缓存。
///
/// 预热属启动期优化，任何一步失败只记告警、降级为「首次命中时再回填」，
/// 不影响其他编码的预热。
async fn warm_one_dict(dict_code: &str) {
    match find_dict_type_by_code(dict_code).await {
        Ok(Some(dict_type)) if dict_type.status == EnableStatus::enabled => {
            match list_enabled_dict_items(&dict_type.id).await {
                Ok(items) => {
                    let list = items.iter().map(DictItemRes::from).collect::<Vec<_>>();
                    match cache_items(dict_code, &list).await {
                        Ok(()) => {
                            tracing::info!(dict_code, count = list.len(), "字典缓存预热完成");
                        }
                        Err(e) => {
                            tracing::warn!(dict_code, error = %e, "字典缓存写入失败，跳过");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(dict_code, error = %e, "读取字典项失败，跳过预热");
                }
            }
        }
        Ok(_) => {}
        Err(e) => {
            tracing::warn!(dict_code, error = %e, "查询字典类型失败，跳过预热");
        }
    }
}
