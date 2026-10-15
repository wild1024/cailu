//! 菜单与权限点的强类型模型。
//!
//! 对应迁移 `V0004__create_sys_menu`。菜单以 `parent_id` 自关联组织为树，
//! `NULL` 表示根节点；`auth_code` 是权限码的唯一来源，供 `#[alun::permission]`
//! 校验使用。
//!
//! # 分层
//!
//! 需要编写 SQL 的树查询集中在本文件；仅调用框架快捷方法（`find_or_none` /
//! `count_by` / `insert` / `update` / `delete` 等）的简单读写由 `handler.rs`
//! 直接执行。

use alun::prelude::*;

use crate::shared::constants::{self, EnableStatus};
use crate::utils;

/// 菜单类型。
///
/// 由旧实现的字符编码（`M` / `C` / `F`）改为 `SMALLINT` 整数判别值，落库更紧凑。
/// `#[int]` 使枚举写 `Value::I32`、读回时兼容任意整数宽度。
///
/// 变体名沿用项目约定的 snake_case。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeMapper)]
#[int]
pub enum MenuType {
    /// 目录（落库值 `1`）：只作分组容器，不对应具体页面。
    directory = 1,
    /// 菜单（落库值 `2`）：对应一个可访问页面。
    menu = 2,
    /// 按钮（落库值 `3`）：页面内的操作权限点，不参与导航渲染。
    button = 3,
}

impl MenuType {
    /// 转换为接口层使用的整数取值。
    ///
    /// # 返回值
    ///
    /// 目录 / 菜单 / 按钮分别返回 `1` / `2` / `3`。
    pub fn as_i16(self) -> i16 {
        match self {
            MenuType::directory => 1,
            MenuType::menu => 2,
            MenuType::button => 3,
        }
    }

    /// 由接口层整数取值还原枚举。
    ///
    /// # 参数
    ///
    /// - `value`：请求传入的菜单类型，仅 `1` / `2` / `3` 合法。
    ///
    /// # 返回值
    ///
    /// 合法值返回对应枚举。
    ///
    /// # 异常情况
    ///
    /// 非 `1` / `2` / `3` 返回 400 参数错误。
    pub fn from_i16(value: i16) -> std::result::Result<Self, ApiError> {
        match value {
            1 => Ok(MenuType::directory),
            2 => Ok(MenuType::menu),
            3 => Ok(MenuType::button),
            _ => Err(ApiError::new(
                400,
                crate::shared::error_code::PARAM_INVALID,
                "菜单类型只能为 1（目录）/ 2（菜单）/ 3（按钮）",
            )),
        }
    }
}

/// `sys_menu` 表对应的强类型模型（菜单 / 权限点）。
///
/// # 树形结构
///
/// `parent_id` 为 `NULL` 时是根节点；层级由 `parent_id` 递归构成，模型层不做
/// 冗余路径存储（与 `sys_dept.dept_path` 不同）。
///
/// # 软删与唯一性
///
/// `deleted_at` 为软删标记；`auth_code` 建立普通索引供权限码检索。
#[derive(Debug, Clone, Model)]
#[table("sys_menu")]
pub struct SysMenuModel {
    /// 主键（UUID v7）。
    pub id: uuid::Uuid,
    /// 菜单 / 按钮名称。
    pub name: String,
    /// 菜单类型（1 目录 / 2 菜单 / 3 按钮）。
    pub menu_type: MenuType,
    /// 权限编码（如 `system:user:list`）。
    pub auth_code: String,
    /// 前端路由地址。
    pub path: Option<String>,
    /// 父节点 ID（`NULL` 为根节点）。
    pub parent_id: Option<uuid::Uuid>,
    /// 前端组件路径。
    pub component: Option<String>,
    /// 排序号，升序展示。
    pub sort_order: i32,
    /// 菜单状态（1 启用 / 0 禁用）。
    pub status: EnableStatus,
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
    /// 隐藏表达式：条件成立时前端隐藏该项（`NULL` 表示不隐藏）。
    pub hidden_expression: Option<String>,
    /// 模块编码（`system` / `support`），用于按域归组。
    pub module_code: Option<String>,
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

// ==================== 数据访问 ====================

/// 菜单树查询条件。
///
/// 由处理器完成参数解析（含菜单类型枚举校验）后传入；`name` 同时匹配名称与权限
/// 编码，模糊匹配由本层统一转换为 `LIKE` 模式。
pub(crate) struct MenuTreeFilter {
    /// 名称 / 权限编码模糊关键词。
    pub name: Option<String>,
    /// 状态过滤（精确匹配）。
    pub status: Option<i16>,
    /// 菜单类型过滤（已校验的强类型枚举）。
    pub menu_type: Option<MenuType>,
    /// 模块编码过滤（精确匹配）。
    pub module_code: Option<String>,
}

/// 按条件查询菜单（扁平列表，供上层组装为树）。
///
/// # 参数
///
/// - `filter`：过滤条件（关键词 / 状态 / 类型 / 模块编码，均可选）。
///
/// # 返回值
///
/// 已按 `sort_order`、`created_at` 升序的扁平菜单列表。
///
/// # 异常情况
///
/// 数据库异常或行映射失败返回 [`DbError`]。
pub(crate) async fn find_menu_list(
    filter: &MenuTreeFilter,
) -> std::result::Result<Vec<SysMenuModel>, DbError> {
    let mut sql = String::from("SELECT * FROM sys_menu WHERE deleted_at IS NULL");
    let mut params: Vec<Value> = Vec::new();

    if let Some(pattern) = utils::like_pattern(filter.name.as_deref()) {
        sql.push_str(" AND (name LIKE ? OR auth_code LIKE ?)");
        params.push(Value::from(pattern.clone()));
        params.push(Value::from(pattern));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        params.push(TypeMapper::to_value(&status));
    }
    if let Some(menu_type) = filter.menu_type {
        sql.push_str(" AND menu_type = ?");
        params.push(TypeMapper::to_value(&menu_type.as_i16()));
    }
    if let Some(module_code) = filter.module_code.as_deref().map(str::trim)
        && !module_code.is_empty()
    {
        sql.push_str(" AND module_code = ?");
        params.push(Value::from(module_code));
    }
    sql.push_str(" ORDER BY sort_order ASC, created_at ASC");

    let rows = db().find_with_params(&sql, params).await?;
    rows.into_iter()
        .map(SysMenuModel::from_row)
        .collect::<std::result::Result<Vec<_>, _>>()
}

// ==================== 菜单树缓存 ====================

/// 读取缓存的全量菜单树。
///
/// 仅缓存「无任何过滤条件」的全量树；命中返回 `Some`，未命中或缓存不可用返回 `None`
/// （由调用方回退查库并回填）。
pub(crate) async fn load_cached_tree() -> Option<Vec<crate::system::menu::dto::MenuRes>> {
    let store = try_cache()?;
    Cache::get(store, constants::CACHE_KEY_MENU_TREE)
        .await
        .ok()
        .flatten()
}

/// 写入全量菜单树缓存。
///
/// 缓存失败只记告警，不影响主流程（缓存仅为读优化）。
pub(crate) async fn cache_tree(nodes: &[crate::system::menu::dto::MenuRes]) {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::set_ex(
        store,
        constants::CACHE_KEY_MENU_TREE,
        nodes,
        constants::CACHE_HOT_QUERY_TTL_SECS,
    )
    .await
    {
        tracing::warn!(error = %e, "写入菜单树缓存失败");
    }
}

/// 失效全量菜单树缓存（菜单写操作后调用）。
pub(crate) async fn invalidate_tree_cache() {
    let Some(store) = try_cache() else {
        return;
    };
    if let Err(e) = Cache::del(store, constants::CACHE_KEY_MENU_TREE).await {
        tracing::warn!(error = %e, "失效菜单树缓存失败");
    }
}
