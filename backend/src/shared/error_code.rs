//! 业务错误码常量。
//!
//! # 与 HTTP 状态码的分工
//!
//! 框架 `alun::codes` 定义的是 **HTTP 语义码**（`OK = 0`、`BAD_REQUEST = 400`、
//! `UNAUTHORIZED = 401`、`FORBIDDEN = 403`、`NOT_FOUND = 404`、`CONFLICT = 409`、
//! `UNPROCESSABLE_ENTITY = 422`、`INTERNAL = 500`、`SERVICE_UNAVAILABLE = 503`），
//! 用于「一个状态码即可表达」的通用场景。
//!
//! 本模块定义的是 **业务错误码**（`1000x` 起分段），用于把「同一 HTTP 状态下
//! 的不同业务语义」区分开：统一响应体中的 `code` 字段取这里的值，HTTP 状态码
//! 仍由处理器显式指定。处理器按「HTTP 状态码 + 业务错误码 + 对外消息」三元组
//! 构造错误：
//!
//! ```ignore
//! use crate::shared::error_code;
//!
//! // 400：参数校验失败
//! return Err(ApiError::new(400, error_code::PARAM_INVALID, "用户名不能为空"));
//! // 401：用户名或密码错误
//! return Err(ApiError::new(401, error_code::AUTH_BAD_CREDENTIALS, "用户名或密码错误"));
//! // 403：无权限
//! return Err(ApiError::new(403, error_code::PERMISSION_DENIED, "权限不足"));
//! ```
//!
//! # 码段划分
//!
//! `0` 表示成功（见 `alun::codes::OK`），不在本模块定义。
//!
//! | 码段 | 含义 |
//! |------|------|
//! | `1000x` | 通用（参数校验、资源不存在、内部错误） |
//! | `2000x` | 认证（凭据错误、令牌过期、账号锁定） |
//! | `3000x` | 授权（无权限、角色 / 菜单 / 部门冲突） |
//! | `4000x` | 支撑域（字典 / 参数 / 文件 / 任务 / 日志） |
//!
//! > 错误码统一在此定义并在处理器中引用，禁止散落魔法数字。

// ==================== 1000x：通用 ====================

/// 参数校验失败（HTTP 400）。
pub const PARAM_INVALID: i32 = 10001;

/// 请求资源不存在（HTTP 404）。
pub const RESOURCE_NOT_FOUND: i32 = 10002;

/// 资源已存在或与现有数据冲突（HTTP 409）。
pub const RESOURCE_CONFLICT: i32 = 10003;

/// 服务器内部错误（HTTP 500）。
pub const INTERNAL_ERROR: i32 = 10004;

/// 请求过于频繁（HTTP 429）。
pub const TOO_MANY_REQUESTS: i32 = 10005;

/// 请求数据格式错误（HTTP 400）。
pub const PAYLOAD_INVALID: i32 = 10006;

// ==================== 2000x：认证 ====================

/// 用户名或密码错误（HTTP 401）。
pub const AUTH_BAD_CREDENTIALS: i32 = 20001;

/// 账号已被禁用（HTTP 403）。
pub const AUTH_ACCOUNT_DISABLED: i32 = 20002;

/// 账号已锁定（HTTP 403）。
pub const AUTH_ACCOUNT_LOCKED: i32 = 20003;

/// 缺少认证令牌（HTTP 401）。
pub const AUTH_TOKEN_MISSING: i32 = 20004;

/// 令牌无效或已过期（HTTP 401）。
pub const AUTH_TOKEN_INVALID: i32 = 20005;

/// 令牌已失效（登出黑名单命中，HTTP 401）。
pub const AUTH_TOKEN_REVOKED: i32 = 20006;

/// 刷新令牌无效或已过期（HTTP 401）。
pub const AUTH_REFRESH_TOKEN_INVALID: i32 = 20007;

// ==================== 3000x：授权 ====================

/// 无权限访问该资源（HTTP 403）。
pub const PERMISSION_DENIED: i32 = 30001;

/// 角色编码已存在（HTTP 409）。
pub const ROLE_CODE_EXISTS: i32 = 30002;

/// 角色不存在（HTTP 404）。
pub const ROLE_NOT_FOUND: i32 = 30003;

/// 角色被内置标识或关联数据占用，不可删除（HTTP 409）。
pub const ROLE_IN_USE: i32 = 30004;

/// 菜单 / 权限点编码冲突（HTTP 409）。
pub const MENU_CODE_CONFLICT: i32 = 30005;

/// 菜单存在子节点，不可删除（HTTP 409）。
pub const MENU_HAS_CHILDREN: i32 = 30006;

/// 部门存在子部门，不可删除（HTTP 409）。
pub const DEPT_HAS_CHILDREN: i32 = 30007;

/// 部门编码已存在（HTTP 409）。
pub const DEPT_CODE_EXISTS: i32 = 30008;

/// 用户不存在（HTTP 404）。
pub const USER_NOT_FOUND: i32 = 30009;

/// 用户名已存在（HTTP 409）。
pub const USERNAME_EXISTS: i32 = 30010;

/// 同级部门下名称已存在（HTTP 409）。
pub const DEPT_NAME_EXISTS: i32 = 30011;

/// 部门不存在（HTTP 404）。
pub const DEPT_NOT_FOUND: i32 = 30012;

/// 角色不存在或已停用（HTTP 404/409，分配角色时校验）。
pub const ROLE_INVALID: i32 = 30013;

// ==================== 4000x：支撑域 ====================

/// 字典编码已存在（HTTP 409）。
pub const DICT_CODE_EXISTS: i32 = 40001;

/// 同一字典下字典项值已存在（HTTP 409）。
pub const DICT_ITEM_VALUE_EXISTS: i32 = 40002;

/// 参数键已存在（HTTP 409）。
pub const CONFIG_KEY_EXISTS: i32 = 40003;

/// 参数不允许修改（HTTP 403）。
pub const CONFIG_NOT_MODIFIABLE: i32 = 40004;

/// 文件不存在（HTTP 404）。
pub const FILE_NOT_FOUND: i32 = 40005;

/// 文件类型不被允许（HTTP 422）。
pub const FILE_TYPE_NOT_ALLOWED: i32 = 40006;

/// 文件大小超出限制（HTTP 422）。
pub const FILE_SIZE_EXCEEDED: i32 = 40007;

/// 定时任务不存在（HTTP 404）。
pub const JOB_NOT_FOUND: i32 = 40008;

/// 定时任务处理器未注册（HTTP 422）。
pub const JOB_TARGET_UNREGISTERED: i32 = 40009;

/// cron 表达式非法（HTTP 422）。
pub const JOB_CRON_INVALID: i32 = 40010;

/// 操作日志不存在（HTTP 404）。
pub const LOG_NOT_FOUND: i32 = 40011;

/// 在线用户会话不存在或已过期（HTTP 404）。
pub const ONLINE_SESSION_NOT_FOUND: i32 = 40012;

/// 在线用户能力依赖缓存，缓存未初始化（HTTP 503）。
pub const ONLINE_CACHE_UNAVAILABLE: i32 = 40013;

/// 未配置参数加密密钥，无法按要求加密存储（HTTP 500）。
pub const CONFIG_SECRET_MISSING: i32 = 40014;

/// 参数值解密失败（密钥不匹配或密文损坏，HTTP 500）。
pub const CONFIG_DECRYPT_FAILED: i32 = 40015;
