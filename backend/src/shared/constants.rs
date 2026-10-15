//! 通用业务常量与共享枚举。
//!
//! 集中定义被多个业务模块复用的取值（分页、状态、内置标识、字典 / 模块编码），
//! 避免各模块散落魔法值。与 HTTP 语义无关的业务错误码见 [`crate::shared::error_code`]。
//!
//! # 软删语义
//!
//! 全部软删表统一使用 `deleted_at TIMESTAMPTZ`（`NULL` = 未删除）。模型侧以
//! `Option<DateTime<Utc>>` 的 `deleted_at` 字段声明后，框架自动完成两件事：
//!
//! - 查询自动追加 `WHERE deleted_at IS NULL`（读取永远看不到已删行）；
//! - `delete` 改写为 `UPDATE ... SET deleted_at = now()` 的标记删除。
//!
//! 需要物理删除的表用 `#[model(hard_delete)]` 关闭该语义。因此业务代码判断
//! 「记录是否存在」应以查询结果为准，**不要**再自行比较 `deleted_at`。

use alun::prelude::*;

// ==================== 分页 ====================

/// 默认分页大小，沿用框架 `PageQuery` 的默认值。
///
/// 分页请求缺省或非法的 `page_size` 会被框架规整到该值。
pub const DEFAULT_PAGE_SIZE: u64 = alun::prelude::DEFAULT_PAGE_SIZE;

/// 单页最大条数，沿用框架 `PageQuery` 的上限。
///
/// 超出该值的 `page_size` 会被规整到该上限，防止一次拉取过多数据。
pub const MAX_PAGE_SIZE: u64 = alun::prelude::MAX_PAGE_SIZE;

// ==================== 状态列取值 ====================

/// 状态列「禁用」取值（`SMALLINT`）。
pub const STATUS_DISABLED: i16 = 0;

/// 状态列「启用」取值（`SMALLINT`）。
pub const STATUS_ENABLED: i16 = 1;

/// 通用启用 / 禁用状态。
///
/// 对应各表的 `status SMALLINT` 列（`sys_user` / `sys_role` / `sys_menu` /
/// `sys_dept` / `sys_dict_type` / `sys_dict_item` / `sys_config` / `sys_file`
/// / `sys_job`）。`#[int]` 使枚举以整数判别值落库，写 `Value::I32`、读回时
/// 兼容任意整数宽度。
///
/// 变体名沿用项目约定的 snake_case。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeMapper)]
#[int]
pub enum EnableStatus {
    /// 禁用（落库值 `0`）。
    disabled = 0,
    /// 启用（落库值 `1`）。
    enabled = 1,
}

impl EnableStatus {
    /// 转换为接口层使用的整数取值。
    ///
    /// # 返回值
    ///
    /// 启用返回 [`STATUS_ENABLED`]，禁用返回 [`STATUS_DISABLED`]。
    pub fn as_i16(self) -> i16 {
        match self {
            EnableStatus::enabled => STATUS_ENABLED,
            EnableStatus::disabled => STATUS_DISABLED,
        }
    }

    /// 由接口层整数取值还原枚举。
    ///
    /// # 参数
    ///
    /// - `value`：请求传入的状态值，仅 `0` / `1` 合法。
    ///
    /// # 返回值
    ///
    /// 合法值返回对应枚举。
    ///
    /// # 异常情况
    ///
    /// 非 `0` / `1` 返回 400 参数错误。
    pub fn from_i16(value: i16) -> std::result::Result<Self, ApiError> {
        match value {
            STATUS_ENABLED => Ok(EnableStatus::enabled),
            STATUS_DISABLED => Ok(EnableStatus::disabled),
            _ => Err(ApiError::new(
                400,
                crate::shared::error_code::PARAM_INVALID,
                "状态值只能为 0（禁用）或 1（启用）",
            )),
        }
    }

    /// 由可选整数取值还原枚举，缺省视为启用。
    ///
    /// 新建 / 更新请求的状态字段可省略，省略时统一按「启用」处理。
    ///
    /// # 参数
    ///
    /// - `raw`：请求传入的状态值，`None` 表示未提供。
    ///
    /// # 返回值
    ///
    /// 未提供时返回 [`EnableStatus::enabled`]，提供时返回对应枚举。
    ///
    /// # 异常情况
    ///
    /// 提供了值但非 `0` / `1` 时返回 400 参数错误。
    pub fn from_optional_i16(raw: Option<i16>) -> std::result::Result<Self, ApiError> {
        match raw {
            Some(value) => Self::from_i16(value),
            None => Ok(EnableStatus::enabled),
        }
    }
}

// ==================== 内置角色与管理员账号 ====================

/// 超级管理员角色编码。
///
/// 模型侧以 `role_code == SUPER_ADMIN_ROLE_CODE` 短路全部权限校验，不查
/// `sys_role_menu`；该角色为内置数据，禁止删除。
pub const SUPER_ADMIN_ROLE_CODE: &str = "SUPER_ADMIN";

/// 内置超级管理员角色名称。
pub const SUPER_ADMIN_ROLE_NAME: &str = "超级管理员";

/// 内置管理员登录账号。
pub const ADMIN_USERNAME: &str = "admin";

/// 管理员初始密码的注入环境变量名。
///
/// 生产环境应通过该环境变量注入初始口令，避免使用内置默认口令。
pub const ADMIN_PASSWORD_ENV: &str = "ALUN_ADMIN_PASSWORD";

/// 管理员初始口令的缺省值。
///
/// 仅在未注入 [`ADMIN_PASSWORD_ENV`] 时使用；启动引导会打印提示日志，
/// 首次登录后必须立即修改。
pub const DEFAULT_ADMIN_PASSWORD: &str = "Admin@123456";

/// [`DEFAULT_ADMIN_PASSWORD`] 的 Argon2id 预置哈希。
///
/// 空库首次启动直接以本常量写入管理员账号，省去现场执行慢哈希
/// （Argon2 默认参数，debug 构建实测约 420ms，是首启第二大头）。
/// 与 pro 版使用相同口令与参数生成，哈希通用。
///
/// 修改 [`DEFAULT_ADMIN_PASSWORD`] 后必须重新生成并同步替换本常量：
/// `cargo run -p alun-utils --example hash_password -- <新口令>`
/// （tests 中的防漂移单测会校验两者一致）。
pub const DEFAULT_ADMIN_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$XGRzRas3GnYicjTY6nAU+Q$IV/OSYNU4cV5IV224Ocar1N1qJDrpaTY15z+4PMMZP0";

// ==================== 登录安全 ====================

/// 连续登录失败达到该次数即锁定账号。
pub const MAX_LOGIN_FAIL_COUNT: i16 = 5;

/// 账号自动解锁的等待时长（分钟）。
///
/// 锁定时记录 `lock_time`，超过该时长后下次登录自动解封。
pub const LOCK_MINUTES: i64 = 30;

/// 登录接口按「用户名 + IP」限流的窗口内最大尝试次数。
pub const LOGIN_RATE_MAX_ATTEMPTS: i64 = 10;

/// 登录限流窗口时长（秒）。
pub const LOGIN_RATE_WINDOW_SECS: u64 = 60;

/// 令牌黑名单 / 登录限流使用的缓存键前缀。
pub const CACHE_KEY_PREFIX_LOGIN: &str = "login:rate:";

// ==================== 在线用户会话 ====================

/// 在线会话缓存键前缀（完整键形如 `online:token:{jti}`）。
pub const CACHE_KEY_PREFIX_ONLINE: &str = "online:token:";

/// 令牌黑名单缓存键前缀。
///
/// 必须与框架 [`alun::prelude::JWT::is_blacklisted`] 使用的键格式保持一致
/// （`token:blacklist:{jti}`），否则强制下线写入的黑名单不会被认证层识别。
pub const CACHE_KEY_PREFIX_TOKEN_BLACKLIST: &str = "token:blacklist:";

/// `sys_config` 中控制 IP 归属地解析的参数键。
pub const CONFIG_KEY_IP_LOCATION: &str = "log.ip_location_enabled";

/// `sys_config` 中控制登录失败锁定阈值的参数键。
pub const CONFIG_KEY_LOGIN_MAX_FAIL: &str = "auth.max_login_fail_count";

/// `sys_config` 中控制账号自动解锁时长的参数键。
pub const CONFIG_KEY_LOGIN_LOCK_MINUTES: &str = "auth.lock_minutes";

/// 参数加密存储使用的密钥注入环境变量名。
///
/// 取值可为任意非空口令字符串：内部统一取其 SHA-256 摘要作为 32 字节密钥
/// （AES-256-GCM）。未注入时，标记 `is_encrypted = true` 的参数写入会失败，
/// 避免敏感值被静默明文落库。
pub const ENV_CONFIG_SECRET_KEY: &str = "X_CONFIG_SECRET_KEY";

/// 加密参数在 `config_value` 中的存储前缀（格式 `enc:v1:{nonce_b64}:{cipher_b64}`）。
pub const CONFIG_CIPHER_PREFIX: &str = "enc:v1:";

// ==================== 分页与查询 ====================

/// 通用列表查询的默认页码。
pub const DEFAULT_PAGE: u64 = 1;

// ==================== 操作日志 ====================

/// 需要记录操作日志的 HTTP 方法（写操作）。
pub const AUDITED_METHODS: [&str; 4] = ["POST", "PUT", "PATCH", "DELETE"];

/// 日志默认保留天数。
///
/// 五张日志表（操作 / 审计 / 认证 / 安全 / 密码变更）的清理任务统一以此为保留期；
/// 操作日志清理接口未传 `days` 时同样取其作为默认值。
pub const OPERATION_LOG_RETENTION_DAYS: i64 = 90;

// ==================== 字典编码 ====================

/// 字典编码：用户状态。
pub const DICT_CODE_USER_STATUS: &str = "user_status";

/// 字典编码：菜单类型。
pub const DICT_CODE_MENU_TYPE: &str = "menu_type";

/// 字典编码：参数类型。
pub const DICT_CODE_CONFIG_TYPE: &str = "config_type";

/// 字典编码：定时任务状态。
pub const DICT_CODE_JOB_STATUS: &str = "job_status";

/// 字典编码：系统参数分类（配置管理页 tab 标签）。
pub const DICT_CODE_CONFIG_CATEGORY: &str = "config_category";

/// 字典编码：认证日志的认证类型（`auth_logs.auth_type`）。
pub const DICT_CODE_LOG_AUTH_TYPE: &str = "log_auth_type";

/// 字典编码：审计 / 操作日志的操作类型（`audit_logs.operation`）。
pub const DICT_CODE_LOG_OPERATION: &str = "log_operation";

/// 字典编码：审计 / 操作日志的目标资源类型（`audit_logs.target_type`）。
pub const DICT_CODE_LOG_TARGET_TYPE: &str = "log_target_type";

/// 字典编码：安全日志的事件类型（`security_logs.event_type`）。
pub const DICT_CODE_LOG_SECURITY_EVENT: &str = "log_security_event";

/// 字典编码：密码变更日志的变更类型（`password_change_logs.change_type`）。
pub const DICT_CODE_LOG_PASSWORD_CHANGE_TYPE: &str = "log_password_change_type";

// ==================== 字典缓存 ====================

/// 字典项缓存键前缀（完整键形如 `dict:items:{dict_code}`）。
///
/// 按字典编码缓存「启用字典项列表」，供高频下拉 / 标签渲染接口命中，避免重复查库。
pub const CACHE_KEY_PREFIX_DICT: &str = "dict:items:";

/// 字典项缓存有效期（秒）。
///
/// 字典项由管理端低频维护，写操作会主动失效对应键，故 TTL 取较长值兜底。
pub const CACHE_DICT_TTL_SECS: u64 = 3600;

/// 启动时预热缓存的字典编码集合。
///
/// 与内置字典种子一致：这些编码被前端基础下拉复用，启动即预热可让首次访问直接命中。
pub const WARM_DICT_CODES: [&str; 10] = [
    DICT_CODE_USER_STATUS,
    DICT_CODE_MENU_TYPE,
    DICT_CODE_CONFIG_TYPE,
    DICT_CODE_JOB_STATUS,
    DICT_CODE_CONFIG_CATEGORY,
    DICT_CODE_LOG_AUTH_TYPE,
    DICT_CODE_LOG_OPERATION,
    DICT_CODE_LOG_TARGET_TYPE,
    DICT_CODE_LOG_SECURITY_EVENT,
    DICT_CODE_LOG_PASSWORD_CHANGE_TYPE,
];

// ==================== 热门查询缓存 ====================

/// 热门下拉 / 树查询缓存的统一有效期（秒）。
///
/// 这些数据由管理端低频维护，写操作会主动失效对应键，TTL 仅作兜底，故取较长值。
pub const CACHE_HOT_QUERY_TTL_SECS: u64 = 3600;

/// 全量部门树缓存键（无过滤条件时缓存，部门写操作后失效）。
///
/// `v2`：`DeptNode` 新增 `leader_name` 字段后升版，避免读到旧结构的缓存。
pub const CACHE_KEY_DEPT_TREE: &str = "dept:tree:all:v2";

/// 全量菜单树缓存键（无过滤条件时缓存，菜单写操作后失效）。
pub const CACHE_KEY_MENU_TREE: &str = "menu:tree:all";

/// 启用角色下拉列表缓存键（角色写操作 / 授权变更后失效）。
pub const CACHE_KEY_ROLE_ENABLED: &str = "role:list:enabled";

// ==================== 权限与策略缓存 ====================

/// 用户角色列表缓存键前缀（完整键形如 `perm:user_roles:{user_id}`）。
///
/// 「角色 / 权限 / 可见菜单」三者构成登录后的主体上下文，每次 `login` / `refresh` /
/// `me` / `menus` 都要读一遍，且权限码查询是 `sys_menu JOIN sys_role_menu JOIN sys_user_role`
/// 三表 JOIN —— 是全系统最高频的读路径。故按用户维度缓存，授权变更后失效。
pub const CACHE_KEY_PREFIX_USER_ROLES: &str = "perm:user_roles:";

/// 用户权限码列表缓存键前缀（完整键形如 `perm:user_perms:{user_id}`）。
pub const CACHE_KEY_PREFIX_USER_PERMS: &str = "perm:user_perms:";

/// 用户可见菜单列表缓存键前缀（完整键形如 `perm:user_menus:{user_id}`）。
pub const CACHE_KEY_PREFIX_USER_MENUS: &str = "perm:user_menus:";

/// 主体上下文缓存的统一有效期（秒）。
///
/// 取较短值（5 分钟）而非热查询的 1 小时：授权变更有写侧失效兜底，但「用户被改角色」
/// 的路径较多，短 TTL 可作为漏失效场景的第二道防线，避免越权窗口过长。
pub const CACHE_PRINCIPAL_TTL_SECS: u64 = 300;

/// 仪表盘聚合指标缓存键（短 TTL，容忍轻微延迟以换取少跑大聚合）。
pub const CACHE_KEY_DASHBOARD_METRICS: &str = "dashboard:metrics";

/// 仪表盘聚合指标缓存有效期（秒）。
///
/// 指标由约十余个子查询聚合而成，前端每次进工作台都会拉取；取 60 秒可在
/// 「数据够新」与「避免反复扫全表」之间取得平衡。
pub const CACHE_DASHBOARD_TTL_SECS: u64 = 60;

/// 公共系统参数缓存键（参数写操作后失效）。
pub const CACHE_KEY_CONFIG_PUBLIC: &str = "config:public";

/// 按键取值的系统参数缓存键前缀（完整键形如 `config:value:{config_key}`）。
pub const CACHE_KEY_PREFIX_CONFIG_VALUE: &str = "config:value:";

/// 系统参数的「变更版本号」键（跨实例同步用）。
///
/// 值为写操作发生时刻的时间戳文本：任一实例完成参数写操作后递增（写入新时间戳），
/// 其余实例由 [`crate::support::config::model::spawn_config_version_watcher`]
/// 轮询比对，发现版本变化即重载本地快照。这样多实例部署时改参数无需等待快照
/// TTL，也无需逐个实例手动调用重载接口。
///
/// 不用 Redis 发布订阅的原因：项目使用的缓存通道只提供 `get` / `set` / `del` /
/// `delete_pattern`，没有订阅能力；版本号轮询每次仅一次 `GET`，开销可忽略。
pub const CACHE_KEY_CONFIG_VERSION: &str = "config:version";

/// 参数版本号轮询间隔（秒）。
///
/// 决定多实例间参数变更的最大可见延迟。取 5 秒：远小于快照 TTL
/// （[`crate::support::config::model::RUNTIME_SNAPSHOT_TTL_SECS`]）的 60 秒，
/// 而每次轮询只有一次 Redis `GET`，对实例与 Redis 的开销都可忽略。
pub const CONFIG_VERSION_POLL_SECS: u64 = 5;

/// 公共（免登录可查）系统参数的键前缀。
///
/// 仅 `config_key` 以该前缀开头的启用参数会被匿名公共接口返回，避免泄露敏感配置。
pub const PUBLIC_CONFIG_KEY_PREFIX: &str = "public.";

// ==================== 模块编码 ====================

/// 模块编码：系统管理域（`sys_menu.module_code`）。
pub const MODULE_CODE_SYSTEM: &str = "system";

/// 模块编码：支撑功能域（`sys_menu.module_code`）。
pub const MODULE_CODE_SUPPORT: &str = "support";

#[cfg(test)]
mod tests {
    use super::*;

    /// 预置哈希必须与默认口令匹配、且拒绝错误口令——防止「改了默认口令
    /// 却忘记重新生成 [`DEFAULT_ADMIN_PASSWORD_HASH`]」的静默漂移
    /// （一旦漂移，默认口令将无法登录）。
    #[test]
    fn default_admin_password_constant_matches_hash() {
        assert!(
            Crypto::verify_password(DEFAULT_ADMIN_PASSWORD, DEFAULT_ADMIN_PASSWORD_HASH)
                .expect("verify default admin password"),
            "DEFAULT_ADMIN_PASSWORD_HASH 与 DEFAULT_ADMIN_PASSWORD 不匹配，请重新生成"
        );
        assert!(
            !Crypto::verify_password("definitely-wrong-password", DEFAULT_ADMIN_PASSWORD_HASH)
                .expect("verify wrong password"),
            "预置哈希未拒绝错误口令"
        );
    }
}
