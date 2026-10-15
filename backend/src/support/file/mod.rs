//! 文件管理子模块。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysFileModel`]（`sys_file`） |
//! | [`dto`] | 文件请求 / 响应 DTO |
//! | [`handler`] | 上传 / 下载 / 分页 / 详情 / 删除接口 |
//!
//! 对外路径前缀 `/api/support/file`。上传 / 下载目录取自 `[upload].path`，
//! 大小上限取自 `[upload].max_size_mb`；相同内容（SHA-256）命中时秒传复用。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | POST | `/api/support/file/upload` | `support:file:upload` | 上传（multipart 字段 `file`，`?category=` 可缺省） |
//! | GET | `/api/support/file/page` | `support:file:list` | 分页查询（`original_name` / `category` / `mime_type` 可选） |
//! | GET | `/api/support/file/{id}` | `support:file:list` | 文件详情 |
//! | DELETE | `/api/support/file/{id}` | `support:file:delete` | 删除（软删元数据 + 尽力删除磁盘文件） |
//! | GET | `/api/support/file/download/{id}` | `support:file:list` | 下载（返回二进制响应，非统一响应体） |

pub mod dto;
pub mod handler;
pub mod model;
