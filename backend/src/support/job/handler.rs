//! 定时任务管理接口：任务定义 CRUD、启停、手动执行与执行日志查询。
//!
//! 处理器承担**参数 / cron / 处理器注册校验、业务编排与响应组装**，并直接调用
//! 框架快捷方法（`find_or_none` / `insert` / `update` / `delete` 等）；需要编写
//! SQL 的查询（分页、名称唯一性校验、启停写库）集中在
//! [`crate::support::job::model`]，动态 SQL 全部走 `?` 占位符参数绑定。
//!
//! 新建 / 更新 / 启停 / 删除任务后统一调用 [`crate::task::scheduler`] 的
//! `reload()` 重新装载调度（框架调度器在 `start()` 时对注册表做一次性快照，
//! 运行期变更必须重载）；重载失败只记错误日志，不影响接口成功。
//!
//! 文件顺序遵循项目约定：**路由方法在前，公共辅助函数在后**。

use alun::prelude::alun_plugin::scheduler::CronSchedule;
use alun::prelude::*;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;

use std::net::SocketAddr;

use crate::shared::constants::EnableStatus;
use crate::shared::error_code;
use crate::support::job::dto::{
    JobLogPageQuery, JobLogRes, JobPageQuery, JobRes, JobSaveReq, JobStatusReq,
};
use crate::support::job::model::{self, SysJobLogModel, SysJobModel};
use crate::support::log;
use crate::utils;

// ==================== 接口 ====================

/// 分页查询定时任务。
///
/// # 参数
///
/// - `query`：过滤条件（名称模糊 / 分组 / 状态，均可选）与分页参数。
///
/// # 返回值
///
/// 成功返回任务分页列表。
///
/// # 异常情况
///
/// 数据库异常返回 500。
#[alun::permission(
    path = "/support/job/page",
    method = "GET",
    permission = "support:job:list"
)]
#[alun::get("/support/job/page")]
pub async fn page(Query(query): Query<JobPageQuery>) -> ResResult<PageData<Vec<JobRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    let filter = model::JobFilter {
        job_name: query.job_name,
        job_group: query.job_group,
        status: query.status,
    };
    let paged = model::page_jobs(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "查询定时任务列表"))?;
    let list = paged.list.iter().map(JobRes::from).collect::<Vec<_>>();

    Ok(Res::page(list, paged.total, paged.page, paged.size))
}

/// 定时任务详情。
///
/// # 参数
///
/// - `id`：任务 ID。
///
/// # 返回值
///
/// 成功返回任务详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：任务不存在。
#[alun::permission(
    path = "/support/job/{id}",
    method = "GET",
    permission = "support:job:list"
)]
#[alun::get("/support/job/{id}")]
pub async fn detail(Path(id): Path<String>) -> ResResult<JobRes> {
    let job = load_job(&id).await?;
    Ok(Res::ok(JobRes::from(&job)))
}

/// 新建定时任务。
///
/// # 参数
///
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（用于审计字段）。
/// - `req`：任务请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`JOB_CREATE`，`after_data` 记录名称 / 分组 / 状态）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回新建任务详情。
///
/// # 异常情况
///
/// - 422：cron 表达式非法或处理器未注册；
/// - 409：任务名称已存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job",
    method = "POST",
    permission = "support:job:create"
)]
#[alun::post("/support/job")]
pub async fn create(
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<JobSaveReq>,
) -> ResResult<JobRes> {
    let job = build_model(&req, None, utils::current_user_uuid(&claims)).await?;
    let mut job = job;
    job.insert(db())
        .await
        .map_err(|e| utils::db_error(e, "新建定时任务"))?;
    reload_scheduler().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "JOB_CREATE",
            "Job",
            job.id.to_string(),
            None,
            Some(serde_json::json!({
                "job_name": job.job_name,
                "job_group": job.job_group,
                "status": job.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(JobRes::from(&job)))
}

/// 更新定时任务（全量）。
///
/// 保留原 `created_at` / `create_by` / `last_run_at` / `next_run_at`，仅更新业务字段。
///
/// # 参数
///
/// - `id`：任务 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `req`：任务请求体。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`JOB_UPDATE`，含名称 / 分组 / 状态的变更前 / 后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回更新后的任务详情。
///
/// # 异常情况
///
/// - 400：ID 或状态非法；
/// - 404：任务不存在；
/// - 422：cron 表达式非法或处理器未注册；
/// - 409：任务名称与其它任务冲突；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job/{id}",
    method = "PUT",
    permission = "support:job:update"
)]
#[alun::put("/support/job/{id}")]
pub async fn update(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<JobSaveReq>,
) -> ResResult<JobRes> {
    let id = utils::require_uuid(&id, "id")?;
    let existing = SysJobModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待更新定时任务"))?
        .ok_or_else(|| ApiError::new(404, error_code::JOB_NOT_FOUND, "定时任务不存在"))?;

    let mut job = build_model(&req, Some(id), utils::current_user_uuid(&claims)).await?;
    // 保留创建审计字段与运行时间，由执行过程维护
    job.created_at = existing.created_at;
    job.create_by = existing.create_by;
    job.last_run_at = existing.last_run_at;
    job.next_run_at = existing.next_run_at;
    job.update(db())
        .await
        .map_err(|e| utils::db_error(e, "更新定时任务"))?;
    reload_scheduler().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "JOB_UPDATE",
            "Job",
            job.id.to_string(),
            Some(serde_json::json!({
                "job_name": existing.job_name,
                "job_group": existing.job_group,
                "status": existing.status.as_i16(),
            })),
            Some(serde_json::json!({
                "job_name": job.job_name,
                "job_group": job.job_group,
                "status": job.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok(JobRes::from(&job)))
}

/// 删除定时任务（软删）。
///
/// # 参数
///
/// - `id`：任务 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`JOB_DELETE`，`before_data` 记录被删任务的名称 / 分组 / 状态）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：任务不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job/{id}",
    method = "DELETE",
    permission = "support:job:delete"
)]
#[alun::delete("/support/job/{id}")]
pub async fn delete(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let job = load_job(&id).await?;
    SysJobModel::of(job.id)
        .delete(db())
        .await
        .map(|_| ())
        .map_err(|e| utils::db_error(e, "删除定时任务"))?;
    reload_scheduler().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "JOB_DELETE",
            "Job",
            job.id.to_string(),
            Some(serde_json::json!({
                "job_name": job.job_name,
                "job_group": job.job_group,
                "status": job.status.as_i16(),
            })),
            None,
        ),
    );
    Ok(Res::ok_empty())
}

/// 启停定时任务。
///
/// # 参数
///
/// - `id`：任务 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明。
/// - `req`：启停请求体（`status`：1 启用 / 0 暂停）。
///
/// # 日志
///
/// 成功后写入 `audit_logs`（`JOB_STATUS`，含名称 / 分组 / 状态的变更前 / 后快照）；
/// 日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 或状态非法；
/// - 404：任务不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job/{id}/status",
    method = "PATCH",
    permission = "support:job:update"
)]
#[alun::patch("/support/job/{id}/status")]
pub async fn change_status(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
    ValidatedJson(req): ValidatedJson<JobStatusReq>,
) -> ResResult<()> {
    let id = utils::require_uuid(&id, "id")?;
    let status = EnableStatus::from_i16(req.status)?;
    let Some(existing) = SysJobModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询待启停定时任务"))?
    else {
        return Err(ApiError::new(
            404,
            error_code::JOB_NOT_FOUND,
            "定时任务不存在",
        ));
    };

    model::update_job_status(&id, status, utils::current_user_uuid(&claims))
        .await
        .map_err(|e| utils::db_error(e, "启停定时任务"))?;
    reload_scheduler().await;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "JOB_STATUS",
            "Job",
            id.to_string(),
            Some(serde_json::json!({
                "job_name": existing.job_name,
                "job_group": existing.job_group,
                "status": existing.status.as_i16(),
            })),
            Some(serde_json::json!({
                "job_name": existing.job_name,
                "job_group": existing.job_group,
                "status": status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok_empty())
}

/// 立即执行一次定时任务（同步等待执行完成）。
///
/// # 参数
///
/// - `id`：任务 ID。
/// - `headers`：请求头（派生日志字段）。
/// - `connect_info`：TCP 连接对端地址（解析真实客户端 IP）。
/// - `claims`：当前用户声明（审计操作者）。
///
/// # 日志
///
/// 执行成功后写入 `audit_logs`（`JOB_TRIGGER`，`after_data` 记录被触发任务的名称 /
/// 分组 / 状态，`before_data` 为空）；日志写入失败不影响业务结果。
///
/// # 返回值
///
/// 成功返回空响应。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：任务不存在；
/// - 422：处理器未注册；
/// - 500：数据库异常或处理器执行失败。
#[alun::permission(
    path = "/support/job/{id}/trigger",
    method = "POST",
    permission = "support:job:trigger"
)]
#[alun::post("/support/job/{id}/trigger")]
pub async fn trigger(
    Path(id): Path<String>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(AuthClaims(claims)): Extension<AuthClaims>,
) -> ResResult<()> {
    let job = load_job(&id).await?;
    crate::task::scheduler().run_job(&job).await?;
    log::record_audit(
        &headers,
        &connect_info,
        log::audit_input(
            &claims,
            "JOB_TRIGGER",
            "Job",
            job.id.to_string(),
            None,
            Some(serde_json::json!({
                "job_name": job.job_name,
                "job_group": job.job_group,
                "status": job.status.as_i16(),
            })),
        ),
    );
    Ok(Res::ok_empty())
}

/// 分页查询任务执行日志。
///
/// # 参数
///
/// - `query`：过滤条件（任务 ID / 名称模糊 / 执行状态，均可选）与分页参数。
///
/// # 返回值
///
/// 成功返回执行日志分页列表。
///
/// # 异常情况
///
/// - 400：`job_id` 格式非法；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job/log/page",
    method = "GET",
    permission = "support:job:list"
)]
#[alun::get("/support/job/log/page")]
pub async fn log_page(Query(query): Query<JobLogPageQuery>) -> ResResult<PageData<Vec<JobLogRes>>> {
    let (page, page_size) = utils::normalize_page(query.page, query.page_size);

    // `job_id` 为可选过滤：提供但格式非法时按参数错误拦截
    let job_id = match query
        .job_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(raw) => Some(utils::require_uuid(raw, "job_id")?),
        None => None,
    };
    let filter = model::JobLogFilter {
        job_id,
        job_name: query.job_name,
        status: query.status,
    };
    let paged = model::page_job_logs(&filter, page, page_size)
        .await
        .map_err(|e| utils::db_error(e, "查询任务执行日志"))?;
    let list = paged.list.iter().map(JobLogRes::from).collect::<Vec<_>>();

    Ok(Res::page(list, paged.total, paged.page, paged.size))
}

/// 任务执行日志详情。
///
/// # 参数
///
/// - `id`：日志 ID。
///
/// # 返回值
///
/// 成功返回日志详情。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：日志不存在；
/// - 500：数据库异常。
#[alun::permission(
    path = "/support/job/log/{id}",
    method = "GET",
    permission = "support:job:list"
)]
#[alun::get("/support/job/log/{id}")]
pub async fn log_detail(Path(id): Path<String>) -> ResResult<JobLogRes> {
    let id = utils::require_uuid(&id, "id")?;
    let log = SysJobLogModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询任务执行日志"))?
        .ok_or_else(|| ApiError::new(404, error_code::LOG_NOT_FOUND, "任务执行日志不存在"))?;
    Ok(Res::ok(JobLogRes::from(&log)))
}

// ==================== 公共辅助 ====================

/// 重新装载调度（新建 / 更新 / 启停 / 删除后调用）。
///
/// 失败只记错误日志，不使业务接口失败。
async fn reload_scheduler() {
    if let Err(e) = crate::task::scheduler().reload().await {
        tracing::error!(error = %e.msg, "重新装载定时任务调度失败");
    }
}

/// 把请求中的 `params` 文本转换为 JSON 值。
///
/// 非空且可解析为 JSON 时存解析结果，否则按 JSON 字符串存；空串视为未提供。
///
/// # 参数
///
/// - `raw`：请求中的 `params` 文本。
///
/// # 返回值
///
/// 空串返回 `None`，否则返回对应的 JSON 值。
fn parse_params(raw: Option<&str>) -> Option<JsonValue> {
    let text = raw?.trim();
    if text.is_empty() {
        return None;
    }
    match serde_json::from_str::<JsonValue>(text) {
        Ok(json) => Some(json),
        Err(_) => Some(JsonValue::String(text.to_string())),
    }
}

/// 按主键读取任务，不存在时返回 404。
///
/// # 参数
///
/// - `raw_id`：路径中的任务 ID 文本。
///
/// # 返回值
///
/// 任务模型。
///
/// # 异常情况
///
/// - 400：ID 格式非法；
/// - 404：任务不存在；
/// - 500：数据库异常。
async fn load_job(raw_id: &str) -> std::result::Result<SysJobModel, ApiError> {
    let id = utils::require_uuid(raw_id, "id")?;
    SysJobModel::find_or_none(db(), &id)
        .await
        .map_err(|e| utils::db_error(e, "查询定时任务"))?
        .ok_or_else(|| ApiError::new(404, error_code::JOB_NOT_FOUND, "定时任务不存在"))
}

/// 从请求构造任务模型字段（新建 / 更新共用）。
///
/// # 参数
///
/// - `req`：任务请求体。
/// - `id`：主键（更新时为既有 ID，新建时为 `None`）。
/// - `operator`：当前操作人 ID。
///
/// # 返回值
///
/// 可直接写入的任务模型（`last_run_at` / `next_run_at` 由调用方决定）。
///
/// # 异常情况
///
/// - 422：cron 表达式非法或处理器未注册；
/// - 409：任务名称已存在；
/// - 400：状态值非法；
/// - 500：数据库异常。
async fn build_model(
    req: &JobSaveReq,
    id: Option<uuid::Uuid>,
    operator: Option<uuid::Uuid>,
) -> std::result::Result<SysJobModel, ApiError> {
    let job_name = req.job_name.trim().to_string();
    let cron_expr = req.cron_expr.trim().to_string();
    let invoke_target = req.invoke_target.trim().to_string();

    // cron 表达式合法性（与调度器同一解析器，避免「能存不能跑」）
    if CronSchedule::parse(&cron_expr).is_err() {
        return Err(ApiError::new(
            422,
            error_code::JOB_CRON_INVALID,
            "cron 表达式非法",
        ));
    }
    // 处理器必须已注册，否则任务永远无法执行
    if !crate::task::is_registered_target(&invoke_target) {
        return Err(ApiError::new(
            422,
            error_code::JOB_TARGET_UNREGISTERED,
            format!("未注册的任务处理器: {invoke_target}"),
        ));
    }
    // 任务名称唯一（软删表，仅比较未删除行）
    let duplicate = model::find_job_by_name(&job_name)
        .await
        .map_err(|e| utils::db_error(e, "校验任务名称唯一性"))?;
    if let Some(existing) = duplicate
        && Some(existing.id) != id
    {
        return Err(ApiError::new(
            409,
            error_code::RESOURCE_CONFLICT,
            "任务名称已存在",
        ));
    }

    let status = match req.status {
        Some(value) => EnableStatus::from_i16(value)?,
        None => EnableStatus::enabled,
    };
    let job_group = req
        .job_group
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("DEFAULT")
        .to_string();
    let now = chrono::Utc::now();
    Ok(SysJobModel {
        id: id.unwrap_or_else(uuid::Uuid::now_v7),
        job_name,
        job_group,
        cron_expr,
        invoke_target,
        params: parse_params(req.params.as_deref()),
        status,
        last_run_at: None,
        next_run_at: None,
        remark: req
            .remark
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        created_at: now,
        updated_at: now,
        create_by: operator,
        update_by: operator,
        deleted_at: None,
    })
}
