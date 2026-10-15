//! 跨模块共享的类型与常量。
//!
//! 存放被多个业务模块复用的定义，避免在各模块重复声明或形成循环引用。
//!
//! | 项 | 内容 | 交付阶段 |
//! |----|------|----------|
//! | [`constants`] | 通用业务常量（状态枚举值、默认分页、超级管理员标识、字典 / 模块编码） | P1 |
//! | [`error_code`] | 业务错误码常量（映射到 `Res::fail` / `ApiError`） | P1 |
//! | [`enums`] | 后端下发枚举的统一词汇层（`EnumItem` / `EnumsRes` / 配色常量） | P1 |

pub mod constants;
pub mod enums;
pub mod error_code;
