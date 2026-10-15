//! 数据字典处理器：字典类型与字典项的增删改查。
//!
//! 处理器承担**参数校验、业务编排与响应组装**，并直接调用框架快捷方法
//! （`find_or_none` / `insert` / `update` / `delete` 等）；需要编写 SQL 的查询
//! （分页、唯一性统计、按编码查询、启用项列表）与缓存逻辑集中在
//! [`crate::support::dict::model`]。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;

use std::net::SocketAddr;

use crate::shared::constants::EnableStatus;
use crate::shared::error_code;
use crate::support::dict::dto::{
    DictItemPageQuery, DictItemRes, DictItemSaveReq, DictTypePageQuery, DictTypeRes,
    DictTypeSaveReq,
};
use crate::support::dict::model::{
    self, DictItemFilter, DictTypeFilter, SysDictItemModel, SysDictTypeModel,
};
use crate::support::log;
use crate::utils;

// ==================== 字典类型接口 ====================

/// 分页查询字典类型。
///
/// # 参数
///
/// - `query`：分页与过滤条件（字典编码 / 名称模糊、状态）。
///
/// # 返回值
///
/// 成功返回字典类型分页数据。
///
/// # 异常情况
///
/// 状态值非法返回 400，数据库异常返回 500。
#[alun::permission(
    path = "/support/dict/type/page",
    method = "GET",
    permission = "support:dict:list"
)]
#[alun::get("/support/dict/type/page")]
pub async fn type_page(
    Query(query): Query<DictTypePageQuery>,
) -> ResResult<PageData<Vec<DictTypeRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);
    let status = query.status.map(EnableStatus::from_i16).transpose()?;
    let filter = DictTypeFilter {
        dict_code: query.dict_code,
        dict_name: query.dict_name,
        status,
    };

    let result = model::page_dict_types(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询字典类型"))?;
    let list = result
        .list
        .iter()
        .map(DictTypeRes::from)
        .collect::<Vec<_>>();
    Ok(Res::page(list, result.total, result.page, result.size))
}

/// 字典类型详情。
///
/// # 参数
///
/// - `id`：字典类型 ID。
///
/// # 返回值
///
/// 成功返回字典类型详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：字典类型不存在。
#[alun::permission(
    path = "/support/dict/type/{id}",
    method = "GET",
    permission = "support:dict:list"
)]
#[alun::get("/support/dict/type/{id}")]
pub async fn type_detail(Path(id): Path<String>) -> ResResult<DictTypeRes> {
    let id = utils::require_uuid(&id, "id")?;
    let dict_type = SysDictTypeModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询字典类型详情"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "字典类型不存在"))?;
    Ok(Res::ok(DictTypeRes::from(&dict_type)))
}

/// 新建字典类型。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（用于审计字段留痕）。
/// - `req`：字典类型请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_TYPE_CREATE`，`after_data` 记录编码 / 名称 / 状态）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建字典类型详情。
///
/// # 异常情况
///
/// - 400：参数非法；
/// - 409：字典编码已存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/type",
    method = "POST",
    permission = "support:dict:create"
)]
#[alun::post("/support/dict/type")]
pub async fn type_create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DictTypeSaveReq>,
) -> ResResult<DictTypeRes> {
    let dict_code = req.dict_code.trim();
    if model::count_dict_code(dict_code, None)
        .await
        .map_err(|e| utils::db_error(e, "校验字典编码唯一性"))?
        > 0
    {
        return Err(ApiError::new(
            409,
            error_code::DICT_CODE_EXISTS,
            "字典编码已存在",
        ));
    }

    let mut dict_type = build_type_model(&req, None, utils::current_user_uuid(&claims))?;
    dict_type
        .insert(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "新建字典类型"))?;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_TYPE_CREATE",
            "DictType",
            dict_type.id.to_string(),
            None,
            Some(serde_json::json!({
                "dict_code": dict_type.dict_code,
                "dict_name": dict_type.dict_name,
                "status": dict_type.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(DictTypeRes::from(&dict_type)))
}

/// 更新字典类型（全量）。
///
/// # 参数
///
/// - `id`：字典类型 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `req`：字典类型请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_TYPE_UPDATE`，含编码 / 名称 / 状态的变更前 / 后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的字典类型详情。
///
/// # 异常情况
///
/// - 400：ID 或状态值非法；
/// - 404：字典类型不存在；
/// - 409：字典编码与他人重复；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/type/{id}",
    method = "PUT",
    permission = "support:dict:update"
)]
#[alun::put("/support/dict/type/{id}")]
pub async fn type_update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DictTypeSaveReq>,
) -> ResResult<DictTypeRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = SysDictTypeModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待更新字典类型"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "字典类型不存在"))?;

    let dict_code = req.dict_code.trim();
    if dict_code != existing.dict_code
        && model::count_dict_code(dict_code, Some(&id))
            .await
            .map_err(|e| utils::db_error(e, "校验字典编码唯一性"))?
            > 0
    {
        return Err(ApiError::new(
            409,
            error_code::DICT_CODE_EXISTS,
            "字典编码已存在",
        ));
    }

    // 保留创建审计字段，仅更新业务字段
    let mut to_save = build_type_model(&req, Some(id), utils::current_user_uuid(&claims))?;
    to_save.created_at = existing.created_at;
    to_save.create_by = existing.create_by;
    to_save
        .update(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "更新字典类型"))?;
    // 编码或状态可能变化，旧、新编码对应的缓存都失效
    model::invalidate_cache(&existing.dict_code).await;
    if to_save.dict_code != existing.dict_code {
        model::invalidate_cache(&to_save.dict_code).await;
    }
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_TYPE_UPDATE",
            "DictType",
            to_save.id.to_string(),
            Some(serde_json::json!({
                "dict_code": existing.dict_code,
                "dict_name": existing.dict_name,
                "status": existing.status.as_i16(),
            })),
            Some(serde_json::json!({
                "dict_code": to_save.dict_code,
                "dict_name": to_save.dict_name,
                "status": to_save.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(DictTypeRes::from(&to_save)))
}

/// 删除字典类型（存在字典项时拒绝）。
///
/// # 参数
///
/// - `id`：字典类型 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_TYPE_DELETE`，`before_data` 记录被删类型的编码 / 名称）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：字典类型不存在；
/// - 409：字典类型下存在字典项；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/type/{id}",
    method = "DELETE",
    permission = "support:dict:delete"
)]
#[alun::delete("/support/dict/type/{id}")]
pub async fn type_delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let Some(dict_type) = SysDictTypeModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待删除字典类型"))?
    else {
        return Err(ApiError::new(
            404,
            error_code::RESOURCE_NOT_FOUND,
            "字典类型不存在",
        ));
    };

    if model::count_items_by_type(&id)
        .await
        .map_err(|e| utils::db_error(e, "统计字典项"))?
        > 0
    {
        return Err(ApiError::new(
            409,
            error_code::RESOURCE_CONFLICT,
            "字典类型下存在字典项，无法删除",
        ));
    }

    SysDictTypeModel::of(id)
        .delete(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "删除字典类型"))?;
    // 类型删除后该编码的字典项缓存立即失效，避免继续下发旧列表
    model::invalidate_cache(&dict_type.dict_code).await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_TYPE_DELETE",
            "DictType",
            id.to_string(),
            Some(serde_json::json!({
                "dict_code": dict_type.dict_code,
                "dict_name": dict_type.dict_name,
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

// ==================== 字典项接口 ====================

/// 分页查询字典项。
///
/// # 参数
///
/// - `query`：分页与过滤条件（所属类型 ID、字典编码、标签模糊、状态）。
///
/// # 返回值
///
/// 成功返回字典项分页数据。
///
/// # 异常情况
///
/// - 400：`dict_type_id` 或状态值非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/item/page",
    method = "GET",
    permission = "support:dict:list"
)]
#[alun::get("/support/dict/item/page")]
pub async fn item_page(
    Query(query): Query<DictItemPageQuery>,
) -> ResResult<PageData<Vec<DictItemRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    let dict_type_id = match query
        .dict_type_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(utils::require_uuid(raw, "dict_type_id")?),
        None => None,
    };
    let status = query.status.map(EnableStatus::from_i16).transpose()?;
    let filter = DictItemFilter {
        dict_type_id,
        dict_code: query
            .dict_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        item_label: query.item_label,
        status,
    };

    let result = model::page_dict_items(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "分页查询字典项"))?;
    let list = result
        .list
        .iter()
        .map(DictItemRes::from)
        .collect::<Vec<_>>();
    Ok(Res::page(list, result.total, result.page, result.size))
}

/// 字典项详情。
///
/// # 参数
///
/// - `id`：字典项 ID。
///
/// # 返回值
///
/// 成功返回字典项详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：字典项不存在。
#[alun::permission(
    path = "/support/dict/item/{id}",
    method = "GET",
    permission = "support:dict:list"
)]
#[alun::get("/support/dict/item/{id}")]
pub async fn item_detail(Path(id): Path<String>) -> ResResult<DictItemRes> {
    let id = utils::require_uuid(&id, "id")?;
    let dict_item = SysDictItemModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询字典项详情"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "字典项不存在"))?;
    Ok(Res::ok(DictItemRes::from(&dict_item)))
}

/// 新建字典项。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（用于审计字段留痕）。
/// - `req`：字典项请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_ITEM_CREATE`，`after_data` 记录所属类型 / 标签 / 值 / 状态）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建字典项详情。
///
/// # 异常情况
///
/// - 400：参数非法；
/// - 404：所属字典类型不存在；
/// - 409：同一字典下字典值已存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/item",
    method = "POST",
    permission = "support:dict:create"
)]
#[alun::post("/support/dict/item")]
pub async fn item_create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DictItemSaveReq>,
) -> ResResult<DictItemRes> {
    let dict_type_id = utils::require_uuid(&req.dict_type_id, "dict_type_id")?;
    ensure_dict_type_exists(&dict_type_id).await?;

    let mut dict_item =
        build_item_model(&req, dict_type_id, None, utils::current_user_uuid(&claims))?;
    if model::count_dict_item_value(&dict_item.dict_type_id, &dict_item.item_value, None)
        .await
        .map_err(|e| utils::db_error(e, "校验字典项值唯一性"))?
        > 0
    {
        return Err(ApiError::new(
            409,
            error_code::DICT_ITEM_VALUE_EXISTS,
            "同一字典下字典值已存在",
        ));
    }
    dict_item
        .insert(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "新建字典项"))?;
    model::invalidate_cache_by_type_id(&dict_item.dict_type_id).await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_ITEM_CREATE",
            "DictItem",
            dict_item.id.to_string(),
            None,
            Some(serde_json::json!({
                "dict_type_id": dict_item.dict_type_id.to_string(),
                "item_label": dict_item.item_label,
                "item_value": dict_item.item_value,
                "status": dict_item.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(DictItemRes::from(&dict_item)))
}

/// 更新字典项（全量）。
///
/// # 参数
///
/// - `id`：字典项 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `req`：字典项请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_ITEM_UPDATE`，含所属类型 / 标签 / 值 / 状态的变更前 / 后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的字典项详情。
///
/// # 异常情况
///
/// - 400：ID 或参数非法；
/// - 404：字典项或所属字典类型不存在；
/// - 409：同一字典下字典值与他人重复；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/item/{id}",
    method = "PUT",
    permission = "support:dict:update"
)]
#[alun::put("/support/dict/item/{id}")]
pub async fn item_update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<DictItemSaveReq>,
) -> ResResult<DictItemRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = SysDictItemModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待更新字典项"))?
        .ok_or_else(|| ApiError::new(404, error_code::RESOURCE_NOT_FOUND, "字典项不存在"))?;

    let dict_type_id = utils::require_uuid(&req.dict_type_id, "dict_type_id")?;
    ensure_dict_type_exists(&dict_type_id).await?;

    let mut to_save = build_item_model(
        &req,
        dict_type_id,
        Some(id),
        utils::current_user_uuid(&claims),
    )?;
    let is_changed =
        to_save.dict_type_id != existing.dict_type_id || to_save.item_value != existing.item_value;
    if is_changed
        && model::count_dict_item_value(&to_save.dict_type_id, &to_save.item_value, Some(&id))
            .await
            .map_err(|e| utils::db_error(e, "校验字典项值唯一性"))?
            > 0
    {
        return Err(ApiError::new(
            409,
            error_code::DICT_ITEM_VALUE_EXISTS,
            "同一字典下字典值已存在",
        ));
    }

    // 保留创建审计字段，仅更新业务字段
    to_save.created_at = existing.created_at;
    to_save.create_by = existing.create_by;
    to_save
        .update(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "更新字典项"))?;
    // 所属类型可能变更，旧、新类型对应的缓存都失效
    model::invalidate_cache_by_type_id(&existing.dict_type_id).await;
    if to_save.dict_type_id != existing.dict_type_id {
        model::invalidate_cache_by_type_id(&to_save.dict_type_id).await;
    }
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_ITEM_UPDATE",
            "DictItem",
            to_save.id.to_string(),
            Some(serde_json::json!({
                "dict_type_id": existing.dict_type_id.to_string(),
                "item_label": existing.item_label,
                "item_value": existing.item_value,
                "status": existing.status.as_i16(),
            })),
            Some(serde_json::json!({
                "dict_type_id": to_save.dict_type_id.to_string(),
                "item_label": to_save.item_label,
                "item_value": to_save.item_value,
                "status": to_save.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(DictItemRes::from(&to_save)))
}

/// 删除字典项。
///
/// # 参数
///
/// - `id`：字典项 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`DICT_ITEM_DELETE`，`before_data` 记录被删项的所属类型 / 标签 / 值）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：字典项不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/dict/item/{id}",
    method = "DELETE",
    permission = "support:dict:delete"
)]
#[alun::delete("/support/dict/item/{id}")]
pub async fn item_delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let Some(dict_item) = SysDictItemModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待删除字典项"))?
    else {
        return Err(ApiError::new(
            404,
            error_code::RESOURCE_NOT_FOUND,
            "字典项不存在",
        ));
    };

    SysDictItemModel::of(id)
        .delete(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "删除字典项"))?;
    model::invalidate_cache_by_type_id(&dict_item.dict_type_id).await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "DICT_ITEM_DELETE",
            "DictItem",
            id.to_string(),
            Some(serde_json::json!({
                "dict_type_id": dict_item.dict_type_id.to_string(),
                "item_label": dict_item.item_label,
                "item_value": dict_item.item_value,
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

/// 按字典编码查询该类型下全部启用字典项（下拉与标签渲染用）。
///
/// 字典类型不存在或已禁用时返回空数组，不视为错误（前端据此静默降级）。
///
/// # 缓存
///
/// 该接口为高频读，采用「缓存优先 + 回填」：命中直接返回；未命中查库后写回缓存。
/// 缓存读写失败只记告警并回退数据库，不影响接口可用性。
///
/// # 参数
///
/// - `dict_code`：字典编码。
///
/// # 返回值
///
/// 成功返回按 `sort_order` 升序的启用字典项列表。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/support/dict/items/{dict_code}",
    method = "GET",
    permission = "support:dict:list"
)]
#[alun::get("/support/dict/items/{dict_code}")]
pub async fn items(Path(dict_code): Path<String>) -> ResResult<Vec<DictItemRes>> {
    // 缓存优先：命中即返回，避免高频下拉重复查库
    match model::load_cached_items(&dict_code).await {
        Ok(Some(cached)) => return Ok(Res::ok(cached)),
        Ok(None) => {}
        Err(e) => tracing::warn!(dict_code, error = %e, "读取字典缓存失败，回退数据库查询"),
    }

    let dict_type = model::find_dict_type_by_code(&dict_code)
        .await
        .map_err(|e| utils::db_error(e, "按编码查询字典类型"))?;
    let Some(dict_type) = dict_type else {
        return Ok(Res::ok(Vec::new()));
    };
    if dict_type.status != EnableStatus::enabled {
        return Ok(Res::ok(Vec::new()));
    }

    let items = model::list_enabled_dict_items(&dict_type.id)
        .await
        .map_err(|e| utils::db_error(e, "按编码查询字典项"))?;
    let list = items.iter().map(DictItemRes::from).collect::<Vec<_>>();
    // 回填缓存（失败不影响本次响应）
    if let Err(e) = model::cache_items(&dict_code, &list).await {
        tracing::warn!(dict_code, error = %e, "回填字典缓存失败");
    }
    Ok(Res::ok(list))
}

// ==================== 公共辅助 ====================

/// 由字典类型请求构造模型字段（新建 / 更新共用）。
///
/// # 参数
///
/// - `req`：字典类型请求体。
/// - `id`：主键（更新时为既有 ID，新建时为 `None`）。
/// - `operator`：当前操作人 ID。
///
/// # 返回值
///
/// 可直接写入的字典类型模型。
///
/// # 异常情况
///
/// 状态值非 0 / 1 返回 400。
fn build_type_model(
    req: &DictTypeSaveReq,
    id: Option<uuid::Uuid>,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<SysDictTypeModel, ApiError> {
    let status = match req.status {
        Some(value) => EnableStatus::from_i16(value)?,
        None => EnableStatus::enabled,
    };
    let now = chrono::Utc::now();
    Ok(SysDictTypeModel {
        id: id.unwrap_or_else(uuid::Uuid::now_v7),
        dict_code: req.dict_code.trim().to_string(),
        dict_name: req.dict_name.trim().to_string(),
        description: req
            .description
            .as_deref()
            .map(str::trim)
            .map(str::to_string),
        status,
        sort_order: req.sort_order.unwrap_or(0),
        remark: req.remark.as_deref().map(str::trim).map(str::to_string),
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    })
}

/// 由字典项请求构造模型字段（新建 / 更新共用，不访问数据库）。
///
/// # 参数
///
/// - `req`：字典项请求体。
/// - `dict_type_id`：已解析并校验存在的所属字典类型 ID。
/// - `id`：主键（更新时为既有 ID，新建时为 `None`）。
/// - `operator`：当前操作人 ID。
///
/// # 返回值
///
/// 可直接写入的字典项模型。
///
/// # 异常情况
///
/// 状态值非 0 / 1 返回 400。
fn build_item_model(
    req: &DictItemSaveReq,
    dict_type_id: uuid::Uuid,
    id: Option<uuid::Uuid>,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<SysDictItemModel, ApiError> {
    let status = match req.status {
        Some(value) => EnableStatus::from_i16(value)?,
        None => EnableStatus::enabled,
    };
    let now = chrono::Utc::now();
    Ok(SysDictItemModel {
        id: id.unwrap_or_else(uuid::Uuid::now_v7),
        dict_type_id,
        item_value: req.item_value.trim().to_string(),
        item_label: req.item_label.trim().to_string(),
        description: req
            .description
            .as_deref()
            .map(str::trim)
            .map(str::to_string),
        sort_order: req.sort_order.unwrap_or(0),
        is_default: req.is_default.unwrap_or(false),
        status,
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    })
}

/// 校验所属字典类型存在，不存在则返回 404。
///
/// # 参数
///
/// - `dict_type_id`：所属字典类型 ID。
///
/// # 返回值
///
/// 存在时返回 `Ok(())`。
///
/// # 异常情况
///
/// - 404：字典类型不存在；
/// - 500：数据库异常。
async fn ensure_dict_type_exists(dict_type_id: &uuid::Uuid) -> std::result::Result<(), ApiError> {
    let exists = SysDictTypeModel::find_or_none(db(), dict_type_id)
        .await
        .map_err(|e| utils::db_error(e, "校验字典类型"))?
        .is_some();
    if !exists {
        return Err(ApiError::new(
            404,
            error_code::RESOURCE_NOT_FOUND,
            "字典类型不存在",
        ));
    }
    Ok(())
}
