//! 操作日志中间件：把写操作（POST / PUT / PATCH / DELETE）落库到 `operation_logs`。
//!
//! # 记录范围
//!
//! 仅记录 [`constants::AUDITED_METHODS`] 中的方法；其它方法（读操作）直接放行，
//! 不产生任何副作用。
//!
//! # 请求体 / 响应体（脱敏）
//!
//! 对 JSON 请求与 JSON 响应，捕获其载荷并**递归脱敏**后分别写入
//! `request_params` / `response_data`（敏感字段见 [`SENSITIVE_FIELDS`]）。脱敏在
//! 请求路径内完成，落库仍由日志总线后台批量执行。
//!
//! 为不破坏原始报文，仅在 `Content-Type` 为 JSON 且载荷长度已知、不超过
//! [`MAX_CAPTURE_BYTES`] 时读取；读取到的字节会**原样回填**请求 / 响应体，二进制、
//! 流式或超限的载荷一律跳过捕获、原样透传。
//!
//! # 非阻塞
//!
//! 响应返回前只把日志模型投递到 [`crate::support::log::bus`]，不等待数据库写入，
//! 因此日志记录不会拖慢接口响应速度。
//!
//! # 客户端 IP
//!
//! 从 `ConnectInfo<SocketAddr>` 取连接对端地址，连同请求头交给
//! [`crate::utils::client_ip`] 解析真实客户端 IP（`X-Real-IP` → `X-Forwarded-For`
//! 末段 → 连接对端 IP），以适配多层代理并防止伪造首段。
//!
//! # 容错
//!
//! 落库失败只记错误日志，绝不改变响应；总线不可用时事件被丢弃并记告警。
//!
//! # 装配
//!
//! 由 [`crate::run`] 通过 `App::layer_from_fn` 调用 [`layer`] 挂载，位于
//! 中间件链最外层，因此可观测到最终响应状态码与真实耗时；操作人身份由认证层
//! 回填到**响应扩展**的 `AuthClaims` 取得（请求进入时尚无声明）。

use std::net::SocketAddr;
use std::time::Instant;

use alun::prelude::*;
use axum::body::{Body, HttpBody};
use axum::extract::{ConnectInfo, Request};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::shared::constants;
use crate::support::log::bus;
use crate::support::log::model::OperationLogModel;
use crate::utils;

// ==================== 脱敏与捕获配置 ====================

/// 需要脱敏的敏感字段名（与字段名大小写不敏感精确匹配，命中后值替换为 `****`）。
///
/// 列表覆盖口令、联系方式、证件号、令牌等常见敏感项；`mask_json_value` 还会
/// 按内容识别手机号 / 身份证 / 邮箱并做部分遮盖。
const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "new_password",
    "old_password",
    "confirm_password",
    "mobile",
    "phone",
    "id_card",
    "idcard",
    "token",
    "access_token",
    "refresh_token",
    "secret",
];

/// 请求 / 响应体捕获上限（字节）。
///
/// 仅当载荷长度已知且不超过该值时读取：既避免大体积载荷占用内存，也确保读取
/// 必然成功，从而可原样回填、不破坏原始报文。
const MAX_CAPTURE_BYTES: usize = 1024 * 1024;

/// 把操作日志中间件挂载到给定路由。
///
/// # 参数
///
/// - `router`：待包装的 axum 路由。
///
/// # 返回值
///
/// 叠加了操作日志层的新路由。
pub fn layer(router: axum::Router) -> axum::Router {
    router.layer(axum::middleware::from_fn(record))
}

/// 操作日志记录中间件本体。
///
/// # 参数
///
/// - `request`：进入的请求。
/// - `next`：后续处理链。
///
/// # 返回值
///
/// 下游处理产生的响应（原样返回，落库失败也不改写）。
pub async fn record(request: Request, next: Next) -> Response {
    // 读操作无需记录，直接放行
    let method = request.method().as_str().to_string();
    if !constants::AUDITED_METHODS.contains(&method.as_str()) {
        return next.run(request).await;
    }

    let started = Instant::now();
    let path = request.uri().path().to_string();
    // 连接对端地址由框架的 into_make_service_with_connect_info 注入
    let connect_info = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0);
    let ip = utils::client_ip(request.headers(), connect_info.as_ref());
    let user_agent = utils::user_agent(request.headers());
    // 优先沿用上游注入的 request_id，缺失时自行生成
    let request_id = utils::request_id(request.headers());
    // 认证层位于本中间件之内：请求进入时尚未注入声明，
    // 认证成功后会回填到**响应扩展**。故两处都尝试，响应扩展优先补齐。
    let mut auth = request.extensions().get::<AuthClaims>().cloned();
    let target_id = parse_target_id(&path);
    let (operation, target_type) = describe_operation(&method, &path);

    // 捕获并脱敏请求体：不满足条件（非 JSON / 长度未知 / 超限）时原样透传
    let (parts, body) = request.into_parts();
    let (request, request_params) = capture_request_body(parts, body).await;

    // 先执行下游，再读取状态码与真实耗时
    let response = next.run(request).await;

    // 捕获并脱敏响应体：不满足条件时原样透传
    let (response, response_data) = capture_response_body(response).await;

    if auth.is_none() {
        auth = response.extensions().get::<AuthClaims>().cloned();
    }
    // 免鉴权接口没有声明，取不到属正常情况
    let (user_id, username) = match auth {
        Some(AuthClaims(claims)) => (utils::current_user_uuid(&claims), claims.username),
        None => (None, None),
    };

    let duration_ms = utils::elapsed_ms(started);
    let status_code = response.status().as_u16() as i32;

    bus::enqueue_operation(OperationLogModel {
        id: uuid::Uuid::now_v7(),
        request_id,
        user_id,
        username,
        operation,
        target_type,
        target_id,
        // 操作详情无结构化来源，保持为空；请求体由 request_params 承载
        detail: None,
        ip,
        user_agent,
        duration_ms: Some(duration_ms),
        request_path: Some(path),
        request_method: Some(method),
        request_params,
        response_data,
        status_code: Some(status_code),
        // 追加写表关闭时间戳 Hook，created_at 必须显式赋值
        created_at: chrono::Utc::now(),
    });

    response
}

/// 捕获并脱敏请求体。
///
/// 仅当 `Content-Type` 为 JSON 且 `Content-Length` 已知、非零且不超过
/// [`MAX_CAPTURE_BYTES`] 时读取；读取到的字节会**原样回填**，其余情况原样透传。
///
/// # 参数
///
/// - `parts`：请求的非主体部分（方法 / 路径 / 头 / 扩展等）。
/// - `body`：请求主体。
///
/// # 返回值
///
/// 重组后的请求与脱敏后的 JSON；未捕获时为 `None`。
async fn capture_request_body(
    parts: axum::http::request::Parts,
    body: Body,
) -> (Request, Option<JsonValue>) {
    let content_length = parts
        .headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if !is_json_content_type(&parts.headers) || !is_capturable_len(content_length) {
        return (Request::from_parts(parts, body), None);
    }

    match axum::body::to_bytes(body, MAX_CAPTURE_BYTES).await {
        Ok(bytes) => {
            let masked = parse_and_mask(&bytes);
            (Request::from_parts(parts, Body::from(bytes)), masked)
        }
        // 读取失败时无法回填原体，放弃捕获以避免内存膨胀（长度已预检，正常不会触发）
        Err(_) => (Request::from_parts(parts, Body::empty()), None),
    }
}

/// 捕获并脱敏响应体。
///
/// 仅当 `Content-Type` 为 JSON 且 `size_hint` 能给出精确长度、非零且不超过
/// [`MAX_CAPTURE_BYTES`] 时读取；读取到的字节会**原样回填**，其余情况原样透传。
///
/// # 参数
///
/// - `response`：下游产生的响应。
///
/// # 返回值
///
/// 重组后的响应与脱敏后的 JSON；未捕获时为 `None`。
async fn capture_response_body(mut response: Response) -> (Response, Option<JsonValue>) {
    let content_length = response.body().size_hint().exact();
    if !is_json_content_type(response.headers()) || !is_capturable_len(content_length) {
        return (response, None);
    }

    // 取出 body 再读取；精确长度已保证读取必然成功，读完原样回填
    let body = std::mem::replace(response.body_mut(), Body::empty());
    match axum::body::to_bytes(body, MAX_CAPTURE_BYTES).await {
        Ok(bytes) => {
            let masked = parse_and_mask(&bytes);
            *response.body_mut() = Body::from(bytes);
            (response, masked)
        }
        // 同上：为避免丢失响应体，捕获失败时保持空体兜底（精确长度已预检，正常不会触发）
        Err(_) => (response, None),
    }
}

/// 判断请求 / 响应载荷是否为 JSON 类型。
///
/// 覆盖 `application/json`、`application/*+json`、`text/json` 等（同时对
/// `; charset=...` 参数不敏感）。
///
/// # 参数
///
/// - `headers`：请求 / 响应头。
///
/// # 返回值
///
/// `Content-Type` 含 `json` 时返回 `true`。
fn is_json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("json"))
}

/// 判断载荷长度是否已知、非零且不超过捕获上限。
///
/// # 参数
///
/// - `len`：载荷长度；未知时为 `None`。
///
/// # 返回值
///
/// 长度落在 `(0, MAX_CAPTURE_BYTES]` 区间时返回 `true`。
fn is_capturable_len(len: Option<u64>) -> bool {
    matches!(len, Some(size) if size > 0 && size <= MAX_CAPTURE_BYTES as u64)
}

/// 解析 JSON 载荷并递归脱敏。
///
/// # 参数
///
/// - `bytes`：载荷字节。
///
/// # 返回值
///
/// 合法 UTF-8 JSON 返回脱敏后的 [`JsonValue`]；否则返回 `None`。
fn parse_and_mask(bytes: &[u8]) -> Option<JsonValue> {
    let value: JsonValue = serde_json::from_slice(bytes).ok()?;
    Some(alun_utils::mask::mask_json_value(value, SENSITIVE_FIELDS))
}

/// 从请求路径中提取操作对象 ID。
///
/// 取路径中第一个形如 UUID 的分段（如 `/api/system/user/{uuid}/roles` 也能识别出
/// `{uuid}`）；路径不含 UUID 时返回 `None`。
///
/// # 参数
///
/// - `path`：请求路径。
///
/// # 返回值
///
/// 命中的对象 ID 字符串；无 UUID 分段时为 `None`。
fn parse_target_id(path: &str) -> Option<String> {
    path.split('/')
        .find(|segment| utils::parse_uuid(segment).is_some())
        .map(str::to_string)
}

/// 依据 HTTP 方法与路径生成人类可读的业务操作描述与对象类型。
///
/// 全局操作日志中间件据此把原始的 `METHOD /path` 转成「新增部门」「保存角色菜单授权」
/// 等业务语义，一处映射即覆盖部门、菜单、角色、用户、配置、字典、上传、任务等全部
/// 模块的写操作，无需在各处理器中散落日志调用。未命中任何已知规则时回退为
/// `METHOD /path`，保证仍可辨识。
///
/// # 参数
///
/// - `method`：HTTP 方法（仅写操作会进入本函数）。
/// - `path`：请求路径（可含 `/api` 前缀，匹配时对其不敏感）。
///
/// # 返回值
///
/// `(操作描述, 对象类型)`；对象类型取资源英文名（`Dept` / `User` / `Role` 等）。
fn describe_operation(method: &str, path: &str) -> (String, Option<String>) {
    // 去掉 /api 前缀后按子串匹配；特殊动作优先于通用 CRUD。
    let p = path.strip_prefix("/api").unwrap_or(path);

    // ── 认证 / 在线会话 ─────────────────────────────────────────────
    if p.contains("/auth/login") {
        return ("账号登录".to_string(), Some("Auth".to_string()));
    }
    if p.contains("/auth/logout") {
        return ("退出登录".to_string(), Some("Auth".to_string()));
    }
    if p.contains("/auth/refresh") {
        return ("刷新令牌".to_string(), Some("Auth".to_string()));
    }
    if p.contains("/support/online/") {
        return ("强制下线用户".to_string(), Some("Online".to_string()));
    }

    // ── 文件 ───────────────────────────────────────────────────────
    if p.contains("/file/upload/batch") {
        return ("批量上传文件".to_string(), Some("File".to_string()));
    }
    if p.contains("/file/upload") {
        return ("上传文件".to_string(), Some("File".to_string()));
    }
    if p.contains("/file/check-duplicate") {
        return ("文件秒传检查".to_string(), Some("File".to_string()));
    }
    if p.contains("/support/file/") {
        return ("删除文件".to_string(), Some("File".to_string()));
    }

    // ── 用户（特殊动作优先） ────────────────────────────────────────
    if p.contains("/system/user/batch-dept") {
        return ("批量调整用户部门".to_string(), Some("User".to_string()));
    }
    if p.contains("/system/user/") && p.ends_with("/roles") {
        return ("分配用户角色".to_string(), Some("User".to_string()));
    }
    if p.contains("/system/user/") && p.ends_with("/password") {
        return ("重置用户密码".to_string(), Some("User".to_string()));
    }
    if p.contains("/system/user/") && p.ends_with("/status") {
        return ("修改用户状态".to_string(), Some("User".to_string()));
    }
    if p.contains("/system/user/") {
        let action = crud_action(method, "用户");
        return (action, Some("User".to_string()));
    }

    // ── 角色 ───────────────────────────────────────────────────────
    if p.contains("/system/role/") && p.ends_with("/menus") {
        return ("保存角色菜单授权".to_string(), Some("Role".to_string()));
    }
    if p.contains("/system/role/") {
        let action = crud_action(method, "角色");
        return (action, Some("Role".to_string()));
    }

    // ── 菜单 ───────────────────────────────────────────────────────
    if p.contains("/system/menu/") {
        let action = crud_action(method, "菜单");
        return (action, Some("Menu".to_string()));
    }

    // ── 部门（迁移并删除优先） ──────────────────────────────────────
    if p.contains("/system/dept/") && p.contains("/migrate") {
        return ("迁移用户并删除部门".to_string(), Some("Dept".to_string()));
    }
    if p.contains("/system/dept") {
        let action = crud_action(method, "部门");
        return (action, Some("Dept".to_string()));
    }

    // ── 系统参数 ───────────────────────────────────────────────────
    if p.contains("/config/refresh-cache") {
        return ("刷新参数缓存".to_string(), Some("Config".to_string()));
    }
    if p.contains("/support/config") {
        let action = crud_action(method, "系统参数");
        return (action, Some("Config".to_string()));
    }

    // ── 字典类型 / 字典项 ──────────────────────────────────────────
    if p.contains("/dict/type") {
        let action = crud_action(method, "字典类型");
        return (action, Some("DictType".to_string()));
    }
    if p.contains("/dict/item") {
        let action = crud_action(method, "字典项");
        return (action, Some("DictItem".to_string()));
    }

    // ── 定时任务（特殊动作优先） ────────────────────────────────────
    if p.contains("/support/job/") && p.ends_with("/trigger") {
        return ("立即执行定时任务".to_string(), Some("Job".to_string()));
    }
    if p.contains("/support/job/") && p.ends_with("/status") {
        return ("修改定时任务状态".to_string(), Some("Job".to_string()));
    }
    if p.contains("/support/job") {
        let action = crud_action(method, "定时任务");
        return (action, Some("Job".to_string()));
    }

    // ── 日志清理 ───────────────────────────────────────────────────
    if p.contains("/support/log/") {
        return ("清理日志".to_string(), Some("Log".to_string()));
    }

    // 兜底：保留原始方法 + 路径，确保未覆盖的写操作仍可辨识
    (format!("{method} {path}"), None)
}

/// 按 HTTP 方法生成通用资源动作描述（新增 / 修改 / 删除）。
///
/// # 参数
///
/// - `method`：HTTP 方法。
/// - `resource`：资源中文名（如「部门」）。
///
/// # 返回值
///
/// `POST` → 新增{资源}，`PUT`/`PATCH` → 修改{资源}，`DELETE` → 删除{资源}，
/// 其它方法回退为「操作{资源}」。
fn crud_action(method: &str, resource: &str) -> String {
    match method {
        "POST" => format!("新增{resource}"),
        "PUT" | "PATCH" => format!("修改{resource}"),
        "DELETE" => format!("删除{resource}"),
        _ => format!("操作{resource}"),
    }
}
