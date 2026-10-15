//! IP 归属地解析。
//!
//! 面向**后台日志 worker**（绝不在请求路径上调用）：把客户端 IP 解析为
//! `国家-省份-城市` 文本，写入 `auth_logs.location`。
//!
//! # 不引入 HTTP 客户端
//!
//! 免费接口 `ip-api.com` 仅提供明文 HTTP，故这里用 `tokio::net::TcpStream`
//! 手写一个最小 HTTP/1.0 请求即可完成调用，避免为一条旁路日志引入
//! `reqwest` 这类重量级依赖（对齐项目「能复用不新增」的依赖约束）。
//!
//! # 开关与缓存
//!
//! - 开关：`sys_config` 的 `log.ip_location_enabled`（缺省启用），由
//!   [`set_enabled`] 在插件启动时快照到进程级原子量，并在管理端改参数后由参数快照
//!   的监听器回调实时刷新（无需重启）；
//! - 缓存：命中结果按 [`CACHE_TTL_SECS`] 缓存到全局 `cache()`，重复 IP 不再外呼；
//! - 失败：任何一步失败都返回 `None`（归属地属可选增强，不影响日志落库）。

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use alun::prelude::*;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 单次外呼的总体超时（含建连与读体）。
const RESOLVE_TIMEOUT: Duration = Duration::from_millis(1000);

/// 归属地缓存有效期（秒）。
const CACHE_TTL_SECS: u64 = 86_400;

/// 免费接口主机（免费额度仅支持 HTTP，故不支持 HTTPS）。
const API_HOST: &str = "ip-api.com";

/// 免费接口端口。
const API_PORT: u16 = 80;

/// 本地 / 内网 IP 的占位归属地。
const LOCAL_LOCATION: &str = "本地网络";

/// 进程级归属地开关（由 [`set_enabled`] 在插件启动时初始化）。
static ENABLED: AtomicBool = AtomicBool::new(true);

/// 判断归属地解析是否启用。
///
/// # 返回值
///
/// 启用返回 `true`；通过 `sys_config` 关闭后返回 `false`。
pub(crate) fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// 设置归属地解析开关（由日志插件在启动时按 `sys_config` 快照，并在参数变更时
/// 由参数快照监听器回调刷新）。
///
/// # 参数
///
/// - `value`：`true` 启用、`false` 关闭。
pub(crate) fn set_enabled(value: bool) {
    ENABLED.store(value, Ordering::Relaxed);
}

/// 仅从缓存读取归属地（不发起外呼）。
///
/// 供**请求路径**使用（如登录时登记在线会话）：命中缓存或本地 IP 时返回结果，
/// 否则立即返回 `None`，绝不因外呼拖慢请求。归属地随后会由日志 worker 补齐到缓存。
///
/// # 参数
///
/// - `ip`：客户端 IP。
///
/// # 返回值
///
/// 命中缓存或本地 IP 时返回 `Some`，否则 `None`。
pub(crate) async fn cached(ip: IpAddr) -> Option<String> {
    let ip_text = ip.to_string();
    if alun_utils::web::is_private_ip(&ip_text) {
        return Some(LOCAL_LOCATION.to_string());
    }
    let store = try_cache()?;
    let key = format!("ipgeo:{ip_text}");
    match Cache::get(store, &key).await {
        Ok(value) => value,
        Err(e) => {
            tracing::debug!(error = %e, ip = %ip_text, "读取 IP 归属地缓存失败");
            None
        }
    }
}

/// 解析 IP 归属地。
///
/// 处理顺序：本地 / 内网 IP 直接返回 [`LOCAL_LOCATION`]；关闭开关时返回 `None`；
/// 命中缓存直接返回；否则外呼免费接口并回写缓存。
///
/// # 参数
///
/// - `ip`：客户端 IP。
///
/// # 返回值
///
/// 解析成功返回 `Some("国家-省份-城市")`（缺省层级自动省略），失败返回 `None`。
pub(crate) async fn resolve(ip: IpAddr) -> Option<String> {
    let ip_text = ip.to_string();
    // 复用 alun_utils 的内网判定，避免重复实现网段规则
    if alun_utils::web::is_private_ip(&ip_text) {
        return Some(LOCAL_LOCATION.to_string());
    }
    if !is_enabled() {
        return None;
    }

    let key = format!("ipgeo:{ip_text}");
    if let Some(store) = try_cache() {
        match Cache::get(store, &key).await {
            Ok(Some(cached)) => return Some(cached),
            Ok(None) => {}
            Err(e) => tracing::debug!(error = %e, ip = %ip_text, "读取 IP 归属地缓存失败"),
        }
    }

    let located = tokio::time::timeout(RESOLVE_TIMEOUT, query_api(&ip_text))
        .await
        .ok()
        .flatten();

    if let (Some(store), Some(value)) = (try_cache(), located.as_ref())
        && let Err(e) = Cache::set_ex(store, &key, value, CACHE_TTL_SECS).await
    {
        tracing::debug!(error = %e, ip = %ip_text, "写入 IP 归属地缓存失败");
    }

    located
}

/// ip-api 返回结构（仅取展示所需字段）。
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    /// 国家。
    country: Option<String>,
    /// 省份 / 地区。
    #[serde(rename = "regionName")]
    region: Option<String>,
    /// 城市。
    city: Option<String>,
}

/// 调用 ip-api 免费接口并组装展示文本。
///
/// # 参数
///
/// - `ip`：IP 文本。
///
/// # 返回值
///
/// 成功且国家非空时返回 `Some(展示文本)`，否则 `None`。
async fn query_api(ip: &str) -> Option<String> {
    let body = http_get(&format!("/json/{ip}?lang=zh-CN")).await?;
    let response: IpApiResponse = serde_json::from_str(&body).ok()?;

    let country = response.country.unwrap_or_default();
    if country.is_empty() {
        return None;
    }
    let region = response.region.unwrap_or_default();
    let city = response.city.unwrap_or_default();
    // 由粗到细拼接，缺失层级自动省略
    let mut parts = vec![country];
    if !region.is_empty() {
        parts.push(region);
    }
    if !city.is_empty() {
        parts.push(city);
    }
    Some(parts.join("-"))
}

/// 手写 HTTP/1.0 GET，读取完整响应体。
///
/// 使用 `Connection: close` + `HTTP/1.0`，服务端响应后即关闭连接，因此
/// 「读到 EOF」等价于「响应体收全」，无需处理 `Content-Length` / chunked。
///
/// # 参数
///
/// - `path`：请求路径（含查询串）。
///
/// # 返回值
///
/// 状态码为 200 时返回响应体文本，其余情况返回 `None`。
async fn http_get(path: &str) -> Option<String> {
    let mut stream = tokio::net::TcpStream::connect((API_HOST, API_PORT))
        .await
        .ok()?;
    let request = format!(
        "GET {path} HTTP/1.0\r\nHost: {API_HOST}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.ok()?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.ok()?;

    let text = String::from_utf8_lossy(&raw);
    let (head, body) = text.split_once("\r\n\r\n")?;
    let status_line = head.lines().next()?;
    if !status_line.contains(" 200") {
        return None;
    }
    Some(body.to_string())
}
