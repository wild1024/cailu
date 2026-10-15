//! 系统参数的强类型模型与数据访问。
//!
//! 对应迁移 `V0009__create_sys_config`。
//!
//! # 分层
//!
//! 需要编写 SQL 的查询（分页查询、按键读取、唯一性统计）与加密 / 解密、类型化
//! 读取等领域行为集中在**本文件**；仅调用框架快捷方法（`find_or_none` /
//! `insert` / `update` / `delete` 等）的简单读写由 `handler.rs` 直接执行。

use alun::prelude::*;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use crate::shared::constants;
use crate::shared::constants::EnableStatus;
use crate::shared::enums::{EnumItem, EnumsRes, color};
use crate::shared::error_code;
use crate::utils;

/// `sys_config` 表对应的强类型模型（系统参数）。
///
/// # 唯一性与软删
///
/// `config_key` 的部分唯一索引带 `WHERE deleted_at IS NULL`。
///
/// # 类型与级别
///
/// - `config_type` 保留为 `String`：取值收敛为 [`config_type`] 白名单四项
///   （`STRING` / `NUMBER` / `BOOLEAN` / `JSON`），写入侧校验、枚举接口下发；
///   此前注释中的 `INT` / `FLOAT` / `BOOL` 与字典种子、前端各说各话，已统一为本口径；
/// - `config_level` 保留为 `i16`（1 系统级 / 2 业务级），仅两级且无需判别值语义，
///   不额外定义枚举。
#[derive(Debug, Clone, Model)]
#[table("sys_config")]
pub struct SysConfigModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 配置键，软删表上的部分唯一列。
    pub config_key: String,
    /// 配置值（文本存储；`is_encrypted` 为真时存密文）。
    pub config_value: Option<String>,
    /// 配置值类型（`STRING` / `INT` / `FLOAT` / `BOOL` / `JSON`）。
    pub config_type: String,
    /// 描述。
    pub description: Option<String>,
    /// 分类。
    pub category: Option<String>,
    /// 排序号，升序展示。
    pub sort_order: i32,
    /// 配置级别（1 系统级 / 2 业务级）。
    pub config_level: i16,
    /// 是否允许在管理端修改。
    pub is_modifiable: bool,
    /// 是否加密存储。
    pub is_encrypted: bool,
    /// 是否需重启生效。
    pub is_restart: bool,
    /// 配置状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
    /// 参考值（提示用）。
    pub ref_value: Option<String>,
    /// 可选值（`值1|标签1,值2|标签2`）。
    pub select_value: Option<String>,
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

impl SysConfigModel {
    /// 读取启用状态参数的文本值。
    ///
    /// 供业务在「参数缺失即回退内置默认值」的场景读取配置：查询失败、记录不存在、
    /// 参数被停用或值为空时统一返回 `None`，由调用方决定兜底值。
    ///
    /// # 读取路径（两级）
    ///
    /// 1. **进程内快照**（[`runtime_snapshot`]）：登录安全策略、IP 归属地开关等
    ///    属请求 / 日志高频路径，逐次查库会给数据库带来无谓压力，因此快照已装载
    ///    且未过期时直接命中内存；
    /// 2. **回源数据库**：快照尚未装载（bootstrap 之前的早期调用）或已过期时查库，
    ///    并顺带把该键回填进快照，避免同一键反复穿透。
    ///
    /// 快照的失效与广播由 [`reload_runtime_snapshot`] 统一负责：参数写操作后调用即
    /// 可让所有读取方（含订阅了开关的模块）实时看到新值，无需重启。
    ///
    /// # 参数
    ///
    /// - `key`：参数键（`sys_config.config_key`）。
    ///
    /// # 返回值
    ///
    /// 参数存在、启用且有非空文本值时返回 `Some(文本)`，否则 `None`。
    pub(crate) async fn read_text(key: &str) -> Option<String> {
        if let Some(snapshot) = runtime_snapshot() {
            // 快照已装载：命中与否都是权威结论，不再回源（未命中即「不存在或已停用」）
            return snapshot.get(key).cloned();
        }
        let value = Self::read_text_from_db(key).await;
        if let Some(raw) = value.as_deref() {
            cache_runtime_value(key, raw);
        }
        value
    }

    /// 直接查库读取启用状态参数的文本值（快照未命中时的回源路径）。
    ///
    /// # 参数
    ///
    /// - `key`：参数键（`sys_config.config_key`）。
    ///
    /// # 返回值
    ///
    /// 参数存在、启用且有非空文本值时返回 `Some(文本)`，否则 `None`。
    async fn read_text_from_db(key: &str) -> Option<String> {
        let found = match Self::find_first_by(db(), "config_key", (key,)).await {
            Ok(Some(record)) => record,
            Ok(None) => return None,
            Err(e) => {
                tracing::warn!(config_key = key, error = %e, "读取系统参数失败，回退默认值");
                return None;
            }
        };
        if found.status != EnableStatus::enabled {
            return None;
        }
        let raw = found.config_value?;
        // 加密存储的参数在此解密；解密失败记告警并回退默认值（本方法语义即「不可用即回退」）
        let raw = if found.is_encrypted {
            match decrypt_config_value(&raw) {
                Ok(value) => value,
                Err(e) => {
                    tracing::warn!(config_key = key, error = ?e, "系统参数解密失败，回退默认值");
                    return None;
                }
            }
        } else {
            raw
        };
        let value = raw.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }

    /// 读取整数型参数。
    ///
    /// 走与 [`Self::read_text`] 相同的两级路径（进程内快照 → 回源数据库）。
    ///
    /// # 参数
    ///
    /// - `key`：参数键（`sys_config.config_key`）。
    ///
    /// # 返回值
    ///
    /// 参数存在、启用且可解析为 `i64` 时返回 `Some`，否则 `None`。
    pub(crate) async fn read_i64(key: &str) -> Option<i64> {
        Self::read_text(key).await?.parse::<i64>().ok()
    }
}

// ==================== 运行时参数快照 ====================
//
// 参考 Alun 框架 `ConfigManager` 的**动态配置**思路（进程内 `RwLock<HashMap>` +
// `set_dynamic` / `get_dynamic`）：框架那份快照只承载 `config.toml`，而本项目的
// 业务参数存在数据库 `sys_config` 里，且会被管理端在运行期修改。因此这里补一层
// 同构的「运行时快照」，解决三个问题：
//
// 1. **读放大**：`read_text` 是登录安全策略、IP 归属地等请求 / 日志高频路径的调用点，
//    逐次 `SELECT` 会给数据库带来无谓压力；快照命中后是纯内存读。
// 2. **运行期不同步**：参数在管理端改完，此前只有重启才会生效（如 `LogPlugin` 启动
//    时把 `log.ip_location_enabled` 快照进进程级原子量，之后再不更新）。
// 3. **多实例 / 直改库**：单实例内的写操作会调 [`reload_runtime_snapshot`] 广播，
//    但别的实例或直接改库的变更需要兜底，因此快照带 TTL，过期后读路径自动回源并
//    顺带刷新新鲜度（自愈，不依赖后台定时任务）。

/// 运行时快照的新鲜期（秒）。
///
/// 写操作（新建 / 更新 / 删除参数、`POST /support/config/reload`）会立即刷新并广播，
/// 因此 TTL 只用于兜底「其它实例改了库」与「直改数据库」两类场景，取值不宜过大。
pub(crate) const RUNTIME_SNAPSHOT_TTL_SECS: u64 = 60;

/// 快照变更监听器：收到新快照后同步回调。
///
/// 回调在刷新方的线程内执行，因此**必须是轻量、无阻塞**的（如写入进程级原子量）。
pub(crate) type RuntimeListener = Arc<dyn Fn(&HashMap<String, String>) + Send + Sync>;

/// 进程内参数快照（键 → 明文值）。
///
/// 只收「启用且值非空」的参数，与 [`SysConfigModel::read_text`] 的可见性口径一致：
/// 快照里没有某个键，等价于该参数不存在 / 已停用，调用方直接回退默认值。
#[derive(Default)]
struct RuntimeSnapshot {
    /// 参数键 → 解密后的明文值。
    values: HashMap<String, String>,
    /// 装载时刻，用于 TTL 判定。
    loaded_at: Option<Instant>,
}

/// 进程级快照容器（`None` = 尚未装载）。
static RUNTIME_SNAPSHOT: OnceLock<RwLock<Option<RuntimeSnapshot>>> = OnceLock::new();

/// 快照变更监听器注册表（监听器名 → 回调）。
static RUNTIME_LISTENERS: OnceLock<RwLock<Vec<(&'static str, RuntimeListener)>>> = OnceLock::new();

/// 取快照容器（首次调用时初始化为空）。
fn runtime_cell() -> &'static RwLock<Option<RuntimeSnapshot>> {
    RUNTIME_SNAPSHOT.get_or_init(|| RwLock::new(None))
}

/// 取监听器注册表（首次调用时初始化为空）。
fn listener_cell() -> &'static RwLock<Vec<(&'static str, RuntimeListener)>> {
    RUNTIME_LISTENERS.get_or_init(|| RwLock::new(Vec::new()))
}

/// 读取当前运行时快照；未装载或已过期时返回 `None`（由调用方回源数据库）。
///
/// # 返回值
///
/// 快照副本；未装载 / 已过期 / 锁中毒时为 `None`。
pub(crate) fn runtime_snapshot() -> Option<HashMap<String, String>> {
    let guard = runtime_cell().read().ok()?;
    let snapshot = guard.as_ref()?;
    let loaded_at = snapshot.loaded_at?;
    if loaded_at.elapsed() >= Duration::from_secs(RUNTIME_SNAPSHOT_TTL_SECS) {
        return None;
    }
    Some(snapshot.values.clone())
}

/// 把单键回填进运行时快照（读路径回源成功时调用，顺带刷新新鲜度）。
///
/// 快照尚未装载时**不**创建空快照：此时只有这一个键的信息，交给
/// [`reload_runtime_snapshot`] 全量装载更正确（避免「半份快照」被当成权威结论）。
fn cache_runtime_value(key: &str, value: &str) {
    let Ok(mut guard) = runtime_cell().write() else {
        return;
    };
    let Some(snapshot) = guard.as_mut() else {
        return;
    };
    snapshot.values.insert(key.to_string(), value.to_string());
    snapshot.loaded_at = Some(Instant::now());
}

/// 注册一个快照变更监听器，并在注册时立即以当前快照回调一次。
///
/// 立即回调的意义：让「注册」（通常是插件启动）之后就能拿到已生效的配置，
/// 不必再各自发起一次读库。
///
/// # 参数
///
/// - `name`：监听器名称（用于日志定位与去重，同名重复注册会覆盖前者）。
/// - `listener`：回调，参数为新的完整快照。
pub(crate) fn subscribe_runtime(name: &'static str, listener: RuntimeListener) {
    if let Ok(mut guard) = listener_cell().write() {
        guard.retain(|(registered, _)| *registered != name);
        guard.push((name, Arc::clone(&listener)));
    }
    // 注册即回调一次：快照未装载时跳过，后续 refresh 会补上
    if let Some(values) = runtime_snapshot() {
        listener(&values);
    }
}

/// 全量重载运行时快照并广播给所有监听器。
///
/// 调用时机：进程启动引导（bootstrap）、参数写操作（新建 / 更新 / 删除）、
/// 以及手动重载接口 `POST /support/config/reload`。
///
/// # 返回值
///
/// 成功返回已装载的参数条数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]，此时**保留原快照**（不因一次失败让运行中的参数全部失效）。
pub(crate) async fn reload_runtime_snapshot() -> std::result::Result<usize, DbError> {
    let rows = SysConfigModel::find_by(
        db(),
        "status = ? AND deleted_at IS NULL ORDER BY sort_order ASC",
        (TypeMapper::to_value(&constants::STATUS_ENABLED),),
    )
    .await?;

    let mut values: HashMap<String, String> = HashMap::with_capacity(rows.len());
    for row in rows {
        let Some(raw) = row.config_value else { continue };
        let plain = if row.is_encrypted {
            match decrypt_config_value(&raw) {
                Ok(value) => value,
                Err(e) => {
                    // 与读路径同口径：解密失败视为不可用，不让单个坏参数拖垮整次装载
                    tracing::warn!(config_key = %row.config_key, error = ?e, "系统参数解密失败，跳过装载");
                    continue;
                }
            }
        } else {
            raw
        };
        let value = plain.trim();
        if value.is_empty() {
            continue;
        }
        values.insert(row.config_key, value.to_string());
    }

    let count = values.len();
    // 先整体替换快照，再取监听器副本广播：回调里若再次读快照，拿到的是新值
    match runtime_cell().write() {
        Ok(mut guard) => {
            *guard = Some(RuntimeSnapshot {
                values,
                loaded_at: Some(Instant::now()),
            });
        }
        Err(_) => {
            tracing::error!("参数快照锁中毒，跳过本次装载");
            return Ok(0);
        }
    }

    let current = runtime_snapshot().unwrap_or_default();
    let listeners: Vec<(&'static str, RuntimeListener)> = listener_cell()
        .read()
        .map(|guard| {
            guard
                .iter()
                .map(|(name, cb)| (*name, Arc::clone(cb)))
                .collect()
        })
        .unwrap_or_default();
    for (name, callback) in listeners {
        callback(&current);
        tracing::debug!(listener = name, keys = count, "参数快照已同步到监听器");
    }
    Ok(count)
}

/// 运行时快照中的参数键列表（诊断 / 重载接口回显用）。
///
/// # 返回值
///
/// 当前快照的键名升序列表；快照未装载或已过期时为空。
pub(crate) fn runtime_snapshot_keys() -> Vec<String> {
    let mut keys = runtime_snapshot().unwrap_or_default().keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

/// 解析布尔型参数文本（`true` / `1` 为真，`false` / `0` 为假）。
///
/// # 参数
///
/// - `raw`：参数文本（大小写不敏感）。
///
/// # 返回值
///
/// 可识别为布尔值时返回 `Some`，否则 `None`。
pub(crate) fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

// ==================== 查询条件 ====================

/// 系统参数分页查询条件。
///
/// 由处理器完成状态值校验与字符串去空白后传入；模糊匹配字段保持原始关键词，
/// 由本层统一转换为 `LIKE` 模式。
pub(crate) struct ConfigFilter {
    /// 配置键（模糊匹配关键词）。
    pub config_key: Option<String>,
    /// 值类型（精确匹配）。
    pub config_type: Option<String>,
    /// 配置分类（精确匹配）。
    pub category: Option<String>,
    /// 状态（调用方已校验为合法枚举值）。
    pub status: Option<EnableStatus>,
}

// ==================== 数据访问 ====================

/// 分页查询系统参数。
///
/// # 参数
///
/// - `filter`：过滤条件（配置键模糊，类型 / 分类精确，状态）。
/// - `page`：页码（从 1 开始）。
/// - `page_size`：每页条数。
///
/// # 返回值
///
/// 已按 `sort_order` 升序的系统参数分页结果。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn page_configs(
    filter: &ConfigFilter,
    page: u64,
    page_size: u64,
) -> std::result::Result<Page<SysConfigModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_config WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();
    if let Some(pattern) = utils::like_pattern(filter.config_key.as_deref()) {
        sql.push_str(" AND config_key LIKE ?");
        params.push(Value::from(pattern));
    }
    if let Some(config_type) = filter.config_type.as_deref() {
        sql.push_str(" AND config_type = ?");
        params.push(Value::from(config_type));
    }
    if let Some(category) = filter.category.as_deref() {
        sql.push_str(" AND category = ?");
        params.push(Value::from(category));
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
        .map(SysConfigModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Page {
        list,
        total,
        page,
        size,
        pages,
    })
}

/// 统计配置键在未删除记录中的数量（唯一性校验）。
///
/// # 参数
///
/// - `config_key`：配置键。
/// - `exclude_id`：需要排除的主键（更新时排除自身）。
///
/// # 返回值
///
/// 命中条数。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn count_config_key(
    config_key: &str,
    exclude_id: Option<&uuid::Uuid>,
) -> std::result::Result<i64, DbError> {
    match exclude_id {
        Some(id) => {
            SysConfigModel::count_by(
                db(),
                "config_key = ? AND deleted_at IS NULL AND id <> ?",
                (config_key, id),
            )
            .await
        }
        None => {
            SysConfigModel::count_by(db(), "config_key = ? AND deleted_at IS NULL", (config_key,))
                .await
        }
    }
}

/// 按配置键查询启用且未删除的系统参数。
///
/// # 参数
///
/// - `config_key`：配置键。
///
/// # 返回值
///
/// 命中返回系统参数模型，未命中返回 `None`。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn find_enabled_config(
    config_key: &str,
) -> std::result::Result<Option<SysConfigModel>, DbError> {
    SysConfigModel::find_first_by(
        db(),
        "config_key = ? AND status = ? AND deleted_at IS NULL",
        (config_key, TypeMapper::to_value(&constants::STATUS_ENABLED)),
    )
    .await
}

// ==================== 公共参数与缓存 ====================

/// 查询所有「公共参数」（`config_key` 以 [`constants::PUBLIC_CONFIG_KEY_PREFIX`]
/// 开头）且启用的记录，按排序值升序。
///
/// 公共参数可匿名访问（如登录页所需的系统标题 / Logo / 版权），以前缀约定的方式
/// 与敏感 / 加密参数隔离，避免在免登录接口泄露其它配置。
///
/// # 返回值
///
/// 公共参数模型列表。
///
/// # 异常情况
///
/// 数据库异常返回 [`DbError`]。
pub(crate) async fn find_public_configs() -> std::result::Result<Vec<SysConfigModel>, DbError> {
    SysConfigModel::find_by(
        db(),
        "config_key LIKE ? AND status = ? AND deleted_at IS NULL ORDER BY sort_order ASC",
        (
            Value::from(format!("{}%", constants::PUBLIC_CONFIG_KEY_PREFIX)),
            TypeMapper::to_value(&constants::STATUS_ENABLED),
        ),
    )
    .await
}

/// 从运行时快照中筛出公共参数（去掉 `public.` 前缀）。
///
/// # 参数
///
/// - `snapshot`：运行时参数快照。
///
/// # 返回值
///
/// 去掉 `public.` 前缀后的键值映射。
pub(crate) fn pick_public(snapshot: &HashMap<String, String>) -> HashMap<String, String> {
    snapshot
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix(constants::PUBLIC_CONFIG_KEY_PREFIX)
                .map(|stripped| (stripped.to_string(), value.clone()))
        })
        .collect()
}

/// 读取缓存的公共参数映射（键为去掉 `public.` 前缀后的配置键）。
///
/// 命中返回 `Some`，未命中或缓存不可用返回 `None`。
pub(crate) async fn load_cached_public() -> Option<HashMap<String, String>> {
    let store = try_cache()?;
    Cache::get::<HashMap<String, String>>(store, constants::CACHE_KEY_CONFIG_PUBLIC)
        .await
        .ok()
        .flatten()
}

/// 写入公共参数缓存；失败只告警，不影响主流程。
pub(crate) async fn cache_public(map: &HashMap<String, String>) {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::set_ex(
        store,
        constants::CACHE_KEY_CONFIG_PUBLIC,
        map,
        constants::CACHE_HOT_QUERY_TTL_SECS,
    )
    .await
    {
        tracing::warn!(error = %e, "写入公共参数缓存失败");
    }
}

/// 读取缓存的按键取值。
///
/// 外层 `Option` 表示缓存是否命中，内层 `Option` 为配置值本身（`None` 即键不存在 /
/// 已停用 / 值为空）。
pub(crate) async fn load_cached_value(key: &str) -> Option<Option<String>> {
    let store = try_cache()?;
    let cache_key = format!("{}{}", constants::CACHE_KEY_PREFIX_CONFIG_VALUE, key);
    Cache::get::<Option<String>>(store, &cache_key)
        .await
        .ok()
        .flatten()
}

/// 写入按键取值缓存；失败只告警，不影响主流程。
pub(crate) async fn cache_value(key: &str, value: &Option<String>) {
    let Some(store) = try_cache() else {
        return;
    };
    let cache_key = format!("{}{}", constants::CACHE_KEY_PREFIX_CONFIG_VALUE, key);
    if let Err(e) = Cache::set_ex(
        store,
        &cache_key,
        value,
        constants::CACHE_HOT_QUERY_TTL_SECS,
    )
    .await
    {
        tracing::warn!(config_key = key, error = %e, "写入参数取值缓存失败");
    }
}

/// 参数写操作后的统一善后：失效缓存 + 刷新进程内快照并广播。
///
/// 参数的读取方分布在三层（Redis 缓存 / 进程内快照 / 数据库），任何一层都可能
/// 残留旧值，因此写操作后必须**同时**处理：
///
/// 1. 失效 Redis 中的公共参数与按键取值缓存（对其它实例也生效）；
/// 2. 全量重载进程内快照并回调监听器（如 IP 归属地开关），使运行期改动立即可见。
///
/// # 异常情况
///
/// 恒不失败：缓存失效与快照重载各自只记告警。写操作本身已落库成功，不应因善后
/// 失败而向调用方返回错误。
pub(crate) async fn after_write() {
    invalidate_config_caches().await;
    if let Err(e) = reload_runtime_snapshot().await {
        tracing::warn!(error = %e, "刷新运行时参数快照失败，将沿用旧快照");
    }
    // 递增版本号让其它实例感知（本实例已完成重载，仅广播给同伴）
    bump_config_version().await;
}

/// 参数版本号的有效期（秒）。
///
/// 版本号只用于跨实例比对，本身无业务含义，取 30 天避免频繁续写；即便键过期被
/// 清掉，监听任务也只会在下一次轮询多重载一次快照，不影响正确性。
const CONFIG_VERSION_TTL_SECS: u64 = 30 * 24 * 3600;

/// 本实例最后一次观测到的参数版本号。
///
/// 用于与 Redis 中的 [`constants::CACHE_KEY_CONFIG_VERSION`] 比对，判断是否需要
/// 重载本地快照。写入方在自身完成写操作后同步更新它，避免把自己刚写的版本当成
/// 「别的实例改了」而重复重载。
static LAST_SEEN_CONFIG_VERSION: OnceLock<RwLock<Option<String>>> = OnceLock::new();

/// 取「最后观测版本」容器（首次调用时初始化为空）。
fn last_seen_cell() -> &'static RwLock<Option<String>> {
    LAST_SEEN_CONFIG_VERSION.get_or_init(|| RwLock::new(None))
}

/// 读取 Redis 中的参数版本号。
///
/// # 返回值
///
/// 版本号文本；缓存不可用或键不存在时返回 `None`。
pub(crate) async fn read_config_version() -> Option<String> {
    let store = try_cache()?;
    Cache::get::<String>(store, constants::CACHE_KEY_CONFIG_VERSION)
        .await
        .ok()
        .flatten()
}

/// 递增参数版本号（参数写操作后调用）。
///
/// 以当前时间戳作为新版本号写入 Redis，供其它实例的监听任务感知变更；同时更新
/// 本实例的「最后观测版本」，避免随后把自己写的值当成外部变更重复重载。
///
/// 缓存不可用时只更新本地值，退化为单实例行为（其余实例靠快照 TTL 兜底）。
pub(crate) async fn bump_config_version() {
    let version = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_default()
        .to_string();
    if let Some(store) = try_cache() {
        if let Err(e) = Cache::set_ex(
            store,
            constants::CACHE_KEY_CONFIG_VERSION,
            &version,
            CONFIG_VERSION_TTL_SECS,
        )
        .await
        {
            tracing::warn!(error = %e, "写入参数版本号失败，其它实例将依赖快照 TTL 兜底");
        }
    }
    if let Ok(mut guard) = last_seen_cell().write() {
        *guard = Some(version);
    }
}

/// 把「最后观测版本」对齐到 Redis 中的当前版本（手动重载后调用）。
///
/// 重载接口已让本实例读到最新值，同步基线可避免监听任务随后把它误判成外部变更
/// 再重载一次。缓存不可用时把基线置空，退化为 TTL 兜底。
pub(crate) async fn sync_version_baseline() {
    let version = read_config_version().await;
    if let Ok(mut guard) = last_seen_cell().write() {
        *guard = version;
    }
}

/// 启动参数版本监听任务（多实例同步）。
///
/// 参数快照是**进程内**的：单实例内写操作会立刻重载，但多实例部署时其它实例无法
/// 感知。本任务按 [`constants::CONFIG_VERSION_POLL_SECS`] 轮询 Redis 中的版本号，
/// 与本地「最后观测版本」不一致即重载快照并广播监听器，把跨实例可见延迟从快照
/// TTL（60 秒）降到轮询间隔（5 秒）。
///
/// 缓存未启用时不启动（退化为 TTL 兜底 + 手动重载接口）。任务永不返回，失败只记
/// 日志，不影响请求处理。
pub(crate) fn spawn_config_version_watcher() {
    if try_cache().is_none() {
        tracing::info!("缓存未启用，跳过参数版本监听（跨实例同步退化为快照 TTL）");
        return;
    }
    tokio::spawn(async move {
        // 先取一次基线，避免刚启动就把「首次读到版本」误判为外部变更
        if let Some(version) = read_config_version().await {
            if let Ok(mut guard) = last_seen_cell().write() {
                *guard = Some(version);
            }
        }
        loop {
            tokio::time::sleep(Duration::from_secs(constants::CONFIG_VERSION_POLL_SECS)).await;
            let remote = read_config_version().await;
            let local = last_seen_cell().read().ok().and_then(|g| g.clone());
            if remote == local {
                continue;
            }
            match reload_runtime_snapshot().await {
                Ok(count) => tracing::info!(params = count, "检测到参数版本变化，已重载运行时快照"),
                Err(e) => {
                    tracing::warn!(error = %e, "参数版本变化后重载快照失败，沿用旧快照")
                }
            }
            if let Ok(mut guard) = last_seen_cell().write() {
                *guard = remote;
            }
        }
    });
}

/// 失效全部参数相关缓存（公共参数 + 所有按键取值），参数写操作后调用。
pub(crate) async fn invalidate_config_caches() {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::del(store, constants::CACHE_KEY_CONFIG_PUBLIC).await {
        tracing::warn!(error = %e, "失效公共参数缓存失败");
    }
    let pattern = format!("{}*", constants::CACHE_KEY_PREFIX_CONFIG_VALUE);
    if let Err(e) = Cache::delete_pattern(store, &pattern).await {
        tracing::warn!(error = %e, "失效参数取值缓存失败");
    }
}

// ==================== 加密存储 ====================

/// 解析参数加密密钥（32 字节）。
///
/// 密钥来自环境变量 [`constants::ENV_CONFIG_SECRET_KEY`]，可为任意非空口令：
/// 统一取其 SHA-256 摘要并十六进制解码为 32 字节，保证长度与确定性。
///
/// # 返回值
///
/// 未注入或为空时返回 `None`，由调用方判定为「未配置密钥」。
fn config_secret_key() -> Option<Vec<u8>> {
    let raw = std::env::var(constants::ENV_CONFIG_SECRET_KEY).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    hex_decode(&alun_utils::Crypto::sha256(raw))
}

/// 十六进制字符串解码为字节序列。
///
/// # 参数
///
/// - `hex`：仅含十六进制字符的字符串，长度必须为偶数。
///
/// # 返回值
///
/// 解码成功返回字节序列；含非法字符或长度为奇数时返回 `None`。
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes = hex.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = (bytes[index] as char).to_digit(16)?;
        let low = (bytes[index + 1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
        index += 2;
    }
    Some(out)
}

/// 加密参数值用于落库。
///
/// 采用 AES-256-GCM，输出格式为 `enc:v1:{nonce_b64}:{cipher_b64}`。
///
/// # 参数
///
/// - `plaintext`：明文参数值。
///
/// # 返回值
///
/// 加密成功返回可存储的密文字符串。
///
/// # 异常情况
///
/// 未配置密钥（[`error_code::CONFIG_SECRET_MISSING`]）或加密失败
/// （[`error_code::INTERNAL_ERROR`]）返回错误，绝不静默明文落库。
pub(crate) fn encrypt_config_value(plaintext: &str) -> std::result::Result<String, ApiError> {
    let key = config_secret_key().ok_or_else(|| {
        ApiError::new(
            500,
            error_code::CONFIG_SECRET_MISSING,
            format!(
                "未配置参数加密密钥（环境变量 {}），无法加密存储",
                constants::ENV_CONFIG_SECRET_KEY
            ),
        )
    })?;
    let (cipher, nonce) = alun_utils::Crypto::aes_encrypt(&key, plaintext)
        .ok_or_else(|| ApiError::new(500, error_code::INTERNAL_ERROR, "参数加密失败"))?;
    Ok(format!(
        "{}{}:{}",
        constants::CONFIG_CIPHER_PREFIX,
        nonce,
        cipher
    ))
}

/// 解密落库的参数值。
///
/// 无 [`constants::CONFIG_CIPHER_PREFIX`] 前缀的历史明文原样返回，以兼容未加密参数。
///
/// # 参数
///
/// - `stored`：`sys_config.config_value` 中的存储值。
///
/// # 返回值
///
/// 解密成功返回明文；无加密前缀时返回原值。
///
/// # 异常情况
///
/// 密文格式非法、未配置密钥或解密失败时返回错误。
pub(crate) fn decrypt_config_value(stored: &str) -> std::result::Result<String, ApiError> {
    let Some(rest) = stored.strip_prefix(constants::CONFIG_CIPHER_PREFIX) else {
        return Ok(stored.to_string());
    };
    let (nonce, cipher) = rest
        .split_once(':')
        .ok_or_else(|| ApiError::new(500, error_code::CONFIG_DECRYPT_FAILED, "参数密文格式非法"))?;
    let key = config_secret_key().ok_or_else(|| {
        ApiError::new(
            500,
            error_code::CONFIG_SECRET_MISSING,
            format!(
                "未配置参数加密密钥（环境变量 {}），无法解密",
                constants::ENV_CONFIG_SECRET_KEY
            ),
        )
    })?;
    alun_utils::Crypto::aes_decrypt(&key, cipher, nonce)
        .ok_or_else(|| ApiError::new(500, error_code::CONFIG_DECRYPT_FAILED, "参数解密失败"))
}

// ==================== 取值白名单 ====================

/// 配置值类型（`sys_config.config_type`）。
///
/// 收敛为白名单常量并经枚举接口下发：此前模型注释（`STRING` / `INT` / `FLOAT` /
/// `BOOL` / `JSON`）、字典种子（`STRING` / `NUMBER` / `BOOLEAN` / `JSON`）与前端
/// 硬编码（`text` / `number` / `boolean` / `json`）三方各说各话，导致种子数据里的
/// `BOOL` 与前端比较的 `boolean` 永不命中、前端新建参数落库 `text` 后端不识别。
/// 现以本模块为唯一口径：**取值大写、四项**（参数值以文本存储，整型/浮型无需拆分）。
pub mod config_type {
    /// 字符串。
    pub const STRING: &str = "STRING";
    /// 数字。
    pub const NUMBER: &str = "NUMBER";
    /// 布尔。
    pub const BOOLEAN: &str = "BOOLEAN";
    /// JSON。
    pub const JSON: &str = "JSON";
}

/// 校验配置值类型取值合法。
///
/// # 参数
///
/// - `kind`：配置值类型。
///
/// # 返回值
///
/// 合法返回 `true`。
pub fn is_valid_config_type(kind: &str) -> bool {
    matches!(
        kind,
        config_type::STRING | config_type::NUMBER | config_type::BOOLEAN | config_type::JSON
    )
}

// ==================== 枚举下发 ====================

/// 本模块枚举表（`GET /support/config/enums`）。
///
/// # 返回值
///
/// - `config_type`：值类型（写入侧经 [`is_valid_config_type`] 白名单校验）。
/// - `category`：参数分类（系统 / 业务 / 日志 / 认证），与种子的 `category` 对应。
pub fn enums() -> EnumsRes {
    crate::module_enums! {
        "config_type" => vec![
            EnumItem::text("字符串", config_type::STRING, color::DEFAULT),
            EnumItem::text("数字", config_type::NUMBER, color::DEFAULT),
            EnumItem::text("布尔", config_type::BOOLEAN, color::DEFAULT),
            EnumItem::text("JSON", config_type::JSON, color::DEFAULT),
        ],
    }
}
