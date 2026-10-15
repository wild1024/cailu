//! 数据字典子模块：字典类型与字典项。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`dto`] | [`dto::DictTypeRes`]、[`dto::DictItemRes`] 等请求 / 响应 DTO |
//! | [`handler`] | 字典类型与字典项的增删改查接口 |
//! | [`model`] | [`model::SysDictTypeModel`]（`sys_dict_type`）、[`model::SysDictItemModel`]（`sys_dict_item`） |
//!
//! 对外路径前缀 `/api/support/dict`。
//!
//! # 接口一览
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/api/support/dict/type/page` | `support:dict:list` | 分页查询字典类型 |
//! | GET | `/api/support/dict/type/{id}` | `support:dict:list` | 字典类型详情 |
//! | POST | `/api/support/dict/type` | `support:dict:create` | 新建字典类型 |
//! | PUT | `/api/support/dict/type/{id}` | `support:dict:update` | 更新字典类型 |
//! | DELETE | `/api/support/dict/type/{id}` | `support:dict:delete` | 删除字典类型 |
//! | GET | `/api/support/dict/item/page` | `support:dict:list` | 分页查询字典项（可按 `dict_code` JOIN 过滤） |
//! | GET | `/api/support/dict/item/{id}` | `support:dict:list` | 字典项详情 |
//! | POST | `/api/support/dict/item` | `support:dict:create` | 新建字典项 |
//! | PUT | `/api/support/dict/item/{id}` | `support:dict:update` | 更新字典项 |
//! | DELETE | `/api/support/dict/item/{id}` | `support:dict:delete` | 删除字典项 |
//! | GET | `/api/support/dict/items/{dict_code}` | `support:dict:list` | 按编码查询全部启用字典项 |
//!
//! 字典类型与字典项均为软删表：`dict_code`、`(dict_type_id, item_value)` 的唯一性
//! 均在「未删除」范围内校验。
//!
//! # 缓存策略
//!
//! `GET /api/support/dict/items/{dict_code}` 为前端下拉高频读口，结果按
//! `dict:items:{dict_code}` 缓存：启动时预热内置编码，读路径缓存优先并回填，
//! 字典类型 / 字典项写操作主动失效对应键（见 [`model`] 的缓存章节）。

pub mod dto;
pub mod handler;
pub mod model;
