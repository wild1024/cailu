//! 系统参数子模块。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`dto`] | [`dto::ConfigRes`]、[`dto::ConfigSaveReq`] 等请求 / 响应 DTO |
//! | [`handler`] | 系统参数的增删改查与按键取值接口 |
//! | [`model`] | [`model::SysConfigModel`]（`sys_config`）、缓存与运行时快照 |
//!
//! 对外路径前缀 `/api/support/config`。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/api/support/config/page` | `support:config:list` | 分页查询系统参数 |
//! | GET | `/api/support/config/{id}` | `support:config:list` | 系统参数详情 |
//! | POST | `/api/support/config` | `support:config:create` | 新建系统参数 |
//! | PUT | `/api/support/config/{id}` | `support:config:update` | 更新系统参数 |
//! | DELETE | `/api/support/config/{id}` | `support:config:delete` | 删除系统参数 |
//! | GET | `/api/support/config/value/{config_key}` | `support:config:list` | 按配置键取启用配置的值 |
//! | GET | `/api/support/config/public` | 免登录 | 公共参数（`public.` 前缀）键值映射 |
//! | GET | `/api/support/config/enums` | `support:config:list` | 值类型枚举（`STRING` / `NUMBER` / `BOOLEAN` / `JSON`） |
//! | POST | `/api/support/config/reload` | `support:config:update` | 重载运行时参数快照 |
//!
//! # 运行期生效机制
//!
//! 参数在管理端改完后**无需重启**即生效，分三层落地（详见 [`model`]）：
//!
//! 1. **Redis 缓存**：写操作后失效公共参数与按键取值缓存（对其它实例同样生效）；
//! 2. **进程内快照**：写操作后全量重载并广播，读路径（登录安全策略、IP 归属地开关、
//!    公共参数等高频接口）直接命中内存，不再逐次查库；
//! 3. **订阅者**：依赖进程级状态的模块（如 [`crate::support::ipgeo`] 的归属地开关）
//!    向快照注册回调，收到新值立即写回原子量。
//!
//! 多实例部署或直改数据库时，用 `POST /api/support/config/reload` 强制本实例重载。
//!
//! # 加密存储说明
//!
//! `is_encrypted = true` 时按 [`constants::ENV_CONFIG_SECRET_KEY`] 派生的
//! AES-256-GCM 密钥加密 `config_value`，读取 / 列表 / 详情 / 按键取值 / 快照装载
//! 时统一解密；未配置密钥时加密写入返回 500，绝不静默明文落库。
//!
//! # 值类型口径
//!
//! `config_type` 取值以 [`model::config_type`] 白名单为唯一事实源
//! （`STRING` / `NUMBER` / `BOOLEAN` / `JSON`），写入侧经 [`model::is_valid_config_type`]
//! 校验，并经 `GET /api/support/config/enums` 下发给前端渲染对应编辑控件。

pub mod dto;
pub mod handler;
pub mod model;
