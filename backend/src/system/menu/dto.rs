//! 菜单 / 权限点的请求与响应 DTO。
//!
//! 同时被 [`crate::system::menu`]（菜单维护）与 [`crate::system::auth`]
//! （当前用户菜单树）复用，避免两处出现结构不一致的树节点定义。

use alun::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::system::menu::model::{MenuType, SysMenuModel};

/// 菜单节点响应（详情与树节点共用）。
///
/// `children` 在「详情」场景为空数组，在「树」场景按 `parent_id` 递归填充。
/// 时间 / 审计字段不下发：菜单是配置型数据，前端不展示。
///
/// 派生 `Deserialize` 以支持菜单树缓存的反序列化回读。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuRes {
    /// 菜单 ID（UUID 字符串）。
    pub id: String,
    /// 菜单 / 按钮名称。
    pub name: String,
    /// 菜单类型（1 目录 / 2 菜单 / 3 按钮）。
    pub menu_type: i16,
    /// 权限编码。
    pub auth_code: String,
    /// 前端路由地址。
    pub path: Option<String>,
    /// 父节点 ID。
    pub parent_id: Option<String>,
    /// 前端组件路径。
    pub component: Option<String>,
    /// 排序号。
    pub sort_order: i32,
    /// 状态（1 启用 / 0 禁用）。
    pub status: i16,
    /// 图标。
    pub icon: Option<String>,
    /// 显示标题。
    pub title: Option<String>,
    /// 是否外链。
    pub is_frame: bool,
    /// 是否缓存页面。
    pub is_cache: bool,
    /// 是否在导航中显示。
    pub is_visible: bool,
    /// 隐藏表达式：条件成立时前端隐藏该项（为空表示不隐藏）。
    pub hidden_expression: Option<String>,
    /// 模块编码。
    pub module_code: Option<String>,
    /// 子节点（树形场景）。
    pub children: Vec<MenuRes>,
}

impl From<&SysMenuModel> for MenuRes {
    /// 由模型构造响应（`children` 初始为空）。
    ///
    /// # 参数
    ///
    /// - `m`：菜单模型引用。
    ///
    /// # 返回值
    ///
    /// 对应的菜单响应对象。
    fn from(m: &SysMenuModel) -> Self {
        Self {
            id: m.id.to_string(),
            name: m.name.clone(),
            menu_type: m.menu_type.as_i16(),
            auth_code: m.auth_code.clone(),
            path: m.path.clone(),
            parent_id: m.parent_id.map(|v| v.to_string()),
            component: m.component.clone(),
            sort_order: m.sort_order,
            status: m.status.as_i16(),
            icon: m.icon.clone(),
            title: m.title.clone(),
            is_frame: m.is_frame,
            is_cache: m.is_cache,
            is_visible: m.is_visible,
            hidden_expression: m.hidden_expression.clone(),
            module_code: m.module_code.clone(),
            children: Vec::new(),
        }
    }
}

/// 菜单新建请求。
#[derive(Debug, Deserialize, Validate)]
pub struct MenuCreateReq {
    /// 菜单 / 按钮名称。
    #[validate(length(min = 1, max = 64, message = "菜单名称长度需为 1-64 个字符"))]
    pub name: String,
    /// 菜单类型（1 目录 / 2 菜单 / 3 按钮）。
    pub menu_type: i16,
    /// 权限编码（按钮必填，目录 / 菜单可为空串）。
    #[validate(length(max = 128, message = "权限编码长度不能超过 128 个字符"))]
    pub auth_code: String,
    /// 前端路由地址。
    #[validate(length(max = 255, message = "路由地址长度不能超过 255 个字符"))]
    pub path: Option<String>,
    /// 父节点 ID（为空表示根节点）。
    pub parent_id: Option<String>,
    /// 前端组件路径。
    #[validate(length(max = 255, message = "组件路径长度不能超过 255 个字符"))]
    pub component: Option<String>,
    /// 排序号。
    pub sort_order: Option<i32>,
    /// 状态（1 启用 / 0 禁用），缺省启用。
    pub status: Option<i16>,
    /// 图标。
    #[validate(length(max = 64, message = "图标长度不能超过 64 个字符"))]
    pub icon: Option<String>,
    /// 显示标题。
    #[validate(length(max = 128, message = "标题长度不能超过 128 个字符"))]
    pub title: Option<String>,
    /// 是否外链。
    pub is_frame: Option<bool>,
    /// 是否缓存页面。
    pub is_cache: Option<bool>,
    /// 是否在导航中显示。
    pub is_visible: Option<bool>,
    /// 隐藏表达式：条件成立时前端隐藏该项（不传表示不隐藏）。
    #[validate(length(max = 255, message = "隐藏表达式长度不能超过 255 个字符"))]
    pub hidden_expression: Option<String>,
    /// 模块编码。
    #[validate(length(max = 32, message = "模块编码长度不能超过 32 个字符"))]
    pub module_code: Option<String>,
}

/// 菜单更新请求（字段与新建一致，PUT 全量更新）。
pub type MenuUpdateReq = MenuCreateReq;

/// 菜单列表 / 树查询参数（全部可选）。
///
/// 与前端契约对齐：关键词参数名为 `name`；为兼容早期调用方，同时接受
/// `keyword` 作为别名。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MenuQuery {
    /// 名称 / 权限编码模糊关键词。
    #[serde(alias = "keyword")]
    pub name: Option<String>,
    /// 状态过滤（1 启用 / 0 禁用）。
    pub status: Option<i16>,
    /// 菜单类型过滤（1 目录 / 2 菜单 / 3 按钮）。
    pub menu_type: Option<i16>,
    /// 模块编码过滤。
    pub module_code: Option<String>,
}

impl MenuQuery {
    /// 把 `menu_type` 可选参数解析为强类型枚举。
    ///
    /// # 返回值
    ///
    /// 未提供时返回 `Ok(None)`；提供但非法时返回 400 参数错误。
    ///
    /// # 异常情况
    ///
    /// `menu_type` 非 1 / 2 / 3 时返回 400。
    pub fn menu_type_enum(&self) -> std::result::Result<Option<MenuType>, ApiError> {
        match self.menu_type {
            Some(v) => MenuType::from_i16(v).map(Some),
            None => Ok(None),
        }
    }
}
