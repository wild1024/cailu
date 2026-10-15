//! 项目内纯工具函数。
//!
//! 只放**无状态、无副作用**的辅助函数（树形结构组装、UUID 值转换、分页规整、
//! 客户端 IP 解析等）。通用工具优先使用框架已提供的 `alun_utils`（字符串、日期、
//! 脱敏、ID、加密、导出等，经 `alun::prelude::*` 直接可用），确实缺失时才在此实现。

use alun::prelude::*;
use std::net::{IpAddr, SocketAddr};

use crate::shared::constants;

/// 解析字符串为 `Uuid`。
///
/// # 参数
///
/// - `s`：UUID 文本（标准带连字符或紧凑格式）。
///
/// # 返回值
///
/// 解析成功返回 `Some(Uuid)`，格式非法返回 `None`。
pub fn parse_uuid(s: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(s.trim()).ok()
}

/// 解析路径 / 查询参数中的 UUID，失败时返回参数错误。
///
/// # 参数
///
/// - `s`：UUID 文本。
/// - `field`：字段名，用于拼接错误消息。
///
/// # 返回值
///
/// 合法 UUID 返回 `Ok(Uuid)`。
///
/// # 异常情况
///
/// 格式非法返回 400 参数错误。
pub fn require_uuid(s: &str, field: &str) -> std::result::Result<uuid::Uuid, ApiError> {
    parse_uuid(s).ok_or_else(|| {
        ApiError::new(
            400,
            crate::shared::error_code::PARAM_INVALID,
            format!("{field} 格式不正确"),
        )
    })
}

/// 从认证声明中解析当前用户 UUID。
///
/// # 参数
///
/// - `claims`：认证中间件注入的令牌声明。
///
/// # 返回值
///
/// `claims.sub` 合法时返回 `Some(Uuid)`；非法（理论上不会发生）返回 `None`，调用方
/// 可据此跳过审计字段写入而不阻断业务。
pub fn current_user_uuid(claims: &TokenClaims) -> Option<uuid::Uuid> {
    parse_uuid(&claims.sub)
}

/// 解析可选的 UUID 参数（未提供或空白视为未传）。
///
/// # 参数
///
/// - `raw`：原始参数值。
/// - `field`：字段名，用于拼接错误消息。
///
/// # 返回值
///
/// 未提供或空白时返回 `Ok(None)`，否则返回解析后的 UUID。
///
/// # 异常情况
///
/// 提供了值但格式非法时返回 400 参数错误。
pub fn parse_optional_uuid(
    raw: Option<&str>,
    field: &str,
) -> std::result::Result<Option<uuid::Uuid>, ApiError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(s) => require_uuid(s, field).map(Some),
    }
}

/// 把可选字符串规整为「去空白后非空」的形式（空白视为未提供）。
///
/// # 参数
///
/// - `raw`：原始字符串。
///
/// # 返回值
///
/// 去空白后仍非空时返回 `Some`，否则返回 `None`。
pub fn trimmed(raw: Option<&str>) -> Option<&str> {
    raw.map(str::trim).filter(|s| !s.is_empty())
}

/// 规整分页参数。
///
/// # 参数
///
/// - `page`：请求页码，缺省用 [`constants::DEFAULT_PAGE`]。
/// - `page_size`：请求每页条数，缺省用 [`constants::DEFAULT_PAGE_SIZE`]。
///
/// # 返回值
///
/// 返回 `(page, page_size)`：`page` 至少为 1，`page_size` 落在 `1..=`[`constants::MAX_PAGE_SIZE`]。
pub fn normalize_page(page: Option<u64>, page_size: Option<u64>) -> (u64, u64) {
    let page = page.unwrap_or(constants::DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(constants::DEFAULT_PAGE_SIZE)
        .clamp(1, constants::MAX_PAGE_SIZE);
    (page, page_size)
}

/// 把可选关键词规整为 LIKE 模式（`%关键词%`）。
///
/// # 参数
///
/// - `keyword`：原始关键词；空白字符串视为未提供。
///
/// # 返回值
///
/// 提供有效关键词时返回 `Some("%关键词%")`，否则返回 `None`。
pub fn like_pattern(keyword: Option<&str>) -> Option<String> {
    keyword
        .map(|k| k.trim())
        .filter(|k| !k.is_empty())
        .map(|k| format!("%{k}%"))
}

/// 从请求头与连接信息解析客户端真实 IP。
///
/// 委托框架 [`alun_utils::extract_client_ip`]，按「可信代理覆写头优先」的顺序解析，
/// 以适配多层代理拓扑，并规避攻击者伪造 `X-Forwarded-For` 首段绕过 IP 限流：
///
/// 1. `X-Real-IP`：由 nginx 覆写为真实客户端 IP，不可伪造；
/// 2. `X-Forwarded-For` **末段**：可信代理追加的真实客户端 IP（**不取首段**）；
/// 3. 连接对端 IP：应用直连（未经代理）场景；
/// 4. 均不可用时框架回退 `0.0.0.0`（内网地址不外泄），此处统一转为 `None`。
///
/// # 参数
///
/// - `headers`：请求头集合。
/// - `connect_info`：TCP 连接对端地址；未接入 `ConnectInfo` 的场景可传 `None`。
///
/// # 返回值
///
/// 解析到有效客户端 IP 时返回 `Some`，否则 `None`。
pub fn client_ip(
    headers: &axum::http::HeaderMap,
    connect_info: Option<&SocketAddr>,
) -> Option<IpAddr> {
    // 未提供连接信息时以未指定地址占位，仅影响「无任何可信头」分支的兜底值
    let fallback = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, 0));
    alun_utils::extract_client_ip(headers, connect_info.unwrap_or(&fallback))
        .parse::<IpAddr>()
        .ok()
        // 过滤 0.0.0.0 / :: 等未指定地址（框架兜底值与内网保护语义）
        .filter(|ip| !ip.is_unspecified())
}

/// 从请求头取请求追踪 ID（优先 `x-request-id`，缺失或空白时生成 UUID v7）。
///
/// 请求 ID 中间件会把同一 ID 写入响应头与日志，这里作为兜底派生入口，
/// 保证处理器拿到的 ID 与链路追踪一致。
///
/// # 参数
///
/// - `headers`：请求头集合。
///
/// # 返回值
///
/// 非空的请求追踪 ID。
pub fn request_id(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string())
}

/// 从请求头取 User-Agent 原文。
///
/// # 参数
///
/// - `headers`：请求头集合。
///
/// # 返回值
///
/// 存在且可转为合法字符串时返回 `Some`，否则 `None`。
pub fn user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

/// 计算自起始时刻以来的毫秒数（饱和到 `i32::MAX`，与日志列的 `INT` 宽度对齐）。
///
/// # 参数
///
/// - `started`：起始时刻。
///
/// # 返回值
///
/// 已耗时的毫秒数。
pub fn elapsed_ms(started: std::time::Instant) -> i32 {
    started.elapsed().as_millis().min(i32::MAX as u128) as i32
}

/// 把「扁平节点列表」组装为树。
///
/// 与具体 DTO 解耦：调用方通过闭包提供主键、父键、排序键与「写回子节点」的动作。
///
/// # 参数
///
/// - `nodes`：扁平节点列表。
/// - `id_of`：取节点主键（字符串形式，用于与父键比对）。
/// - `parent_of`：取节点父键；`None` 或空串视为根节点。
/// - `sort_of`：取排序号，同级按升序排列。
/// - `set_children`：把已组装好的子节点写回父节点。
///
/// # 返回值
///
/// 根节点列表（已递归填充 `children`）。
pub fn build_tree<T, FId, FPid, FSort, FSet>(
    nodes: Vec<T>,
    id_of: FId,
    parent_of: FPid,
    sort_of: FSort,
    set_children: FSet,
) -> Vec<T>
where
    T: Clone,
    FId: Fn(&T) -> String,
    FPid: Fn(&T) -> Option<String>,
    FSort: Fn(&T) -> i32,
    FSet: Fn(&mut T, Vec<T>),
{
    let mut children_map: std::collections::HashMap<String, Vec<T>> =
        std::collections::HashMap::new();
    let mut roots: Vec<T> = Vec::new();

    for node in nodes {
        match parent_of(&node) {
            Some(pid) if !pid.is_empty() => children_map.entry(pid).or_default().push(node),
            _ => roots.push(node),
        }
    }

    fn attach<T, FId, FSort, FSet>(
        node: &mut T,
        children_map: &std::collections::HashMap<String, Vec<T>>,
        id_of: &FId,
        sort_of: &FSort,
        set_children: &FSet,
    ) where
        T: Clone,
        FId: Fn(&T) -> String,
        FSort: Fn(&T) -> i32,
        FSet: Fn(&mut T, Vec<T>),
    {
        let id = id_of(node);
        let mut children = children_map.get(&id).cloned().unwrap_or_default();
        children.sort_by_key(|c| sort_of(c));
        for child in &mut children {
            attach(child, children_map, id_of, sort_of, set_children);
        }
        set_children(node, children);
    }

    roots.sort_by_key(|r| sort_of(r));
    for root in &mut roots {
        attach(root, &children_map, &id_of, &sort_of, &set_children);
    }
    roots
}

/// 把数据库错误映射为统一的 500 内部错误（对外隐藏细节，细节写入日志）。
///
/// # 参数
///
/// - `e`：数据库错误。
/// - `ctx`：业务上下文描述，用于日志定位。
///
/// # 返回值
///
/// 统一响应体为 `code = INTERNAL_ERROR`、消息为「服务器内部错误」的 [`ApiError`]。
pub fn db_error(e: DbError, ctx: &str) -> ApiError {
    tracing::error!(context = ctx, error = %e, "数据库操作失败");
    // 对外仍是「HTTP 500 + 业务码 10004」，保持本项目错误码体系一致；
    // 真实原因写进 `internal_detail`，交由 `ApiError` 的 `IntoResponse` 统一记日志，
    // 绝不出现在响应体里（与 alun 0.3 的错误收敛惯例一致）。
    ApiError {
        status: 500,
        code: crate::shared::error_code::INTERNAL_ERROR,
        msg: "服务器内部错误".to_string(),
        internal_detail: Some(format!("{ctx}: {e}")),
    }
}

/// 唯一性校验的收口：命中行数大于 0 即返回 409 冲突错误。
///
/// 供各模块的「编码 / 账号唯一性校验」复用，统一冲突错误的构造方式。
///
/// # 参数
///
/// - `count`：唯一性校验查询命中的行数。
/// - `code`：冲突时返回的业务错误码。
/// - `message`：冲突时的提示文案。
///
/// # 返回值
///
/// `count == 0` 时返回 `Ok(())`。
///
/// # 异常情况
///
/// `count > 0` 时返回 409 冲突错误。
pub fn ensure_unique(count: i64, code: i32, message: &str) -> std::result::Result<(), ApiError> {
    if count > 0 {
        return Err(ApiError::new(409, code, message));
    }
    Ok(())
}
