//! 后端下发枚举：各模块 `GET /{system|support}/{module}/enums` 的统一词汇层。
//!
//! # 为什么由后端下发
//!
//! 枚举的「取值 → 中文名 → 标签配色」三件套若前后端各写一份，必然漂移。本仓已有
//! 前车之鉴：`sys_config.config_type` 的取值在模型注释（`STRING` / `INT` / `FLOAT` /
//! `BOOL` / `JSON`）、字典种子（`STRING` / `NUMBER` / `BOOLEAN` / `JSON`）与前端硬编码
//! （`text` / `number` / `boolean` / `json`）之间存在**三重不一致**，导致种子数据里
//! 的 `BOOL` 与前端比较的 `boolean` 互不命中。因此约定**取值与文案的单一事实源放在后端**：
//! 枚举定义紧邻它所描述的领域类型（`model.rs` 的常量模块），经本模块统一编码后下发；
//! 前端只做「按模块取一次 → 缓存 → 渲染」。
//!
//! # 响应结构
//!
//! 返回「枚举名 → 枚举项数组」的对象，与 `sys_dict` 的前端投影保持一致的三字段
//! （`label` / `value` / `color`），使前端可以用同一套渲染逻辑（`DictTag` 一族）
//! 处理字典与枚举；`description` 为可选扩展，供单选卡片一类的组件补充说明文案。
//!
//! ```json
//! {
//!   "code": 0,
//!   "data": {
//!     "status": [
//!       { "label": "草稿", "value": 0, "color": "default" },
//!       { "label": "已发布", "value": 1, "color": "success" }
//!     ]
//!   }
//! }
//! ```
//!
//! # `value` 保留原生类型
//!
//! `value` 不做字符串化：整数枚举下发 JSON number，字符串枚举下发 JSON string。
//! 这样前端把枚举项直接塞进 `t-select` 的 `options` 时，回填进表单的值与接口契约
//! （`status: i16` 之类）天然一致，无需在每个调用点写 `Number(...)` 转换——而
//! 「字符串值 vs 数值字段」的隐式转换恰恰是这类漂移最容易被掩盖的地方。
//!
//! # 权限
//!
//! 枚举不含业务数据，但也不应无条件公开：各模块的 `enums` 路由统一挂该模块的
//! `{domain}:{module}:list` 权限码，与列表接口同权限——能看列表的人本就该看到筛选
//! 下拉与标签配色，无权访问该模块的人也不需要它的枚举。

use alun::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;

/// 标签配色常量（对应前端 TDesign `t-tag` 的 `theme`）。
///
/// 取值必须落在前端 `TagTheme` 的联合类型内，否则标签会静默回退为默认样式。
pub mod color {
    /// 中性（默认）。
    pub const DEFAULT: &str = "default";
    /// 主色（进行中 / 信息类）。
    pub const PRIMARY: &str = "primary";
    /// 成功 / 正向。
    pub const SUCCESS: &str = "success";
    /// 警告 / 过渡态。
    pub const WARNING: &str = "warning";
    /// 危险 / 负向。
    pub const DANGER: &str = "danger";
}

/// 单个枚举项。
///
/// 字段与 `sys_dict_item` 的前端投影对齐（`label` / `value` / `color`），
/// 使前端可用同一套组件渲染。
#[derive(Debug, Clone, Serialize)]
pub struct EnumItem {
    /// 展示文案（中文名）。
    pub label: &'static str,
    /// 取值，保留原生 JSON 类型（整数枚举为 number，编码枚举为 string）。
    pub value: JsonValue,
    /// 标签配色。
    pub color: &'static str,
    /// 补充说明（可选），供单选项的辅助文案使用；缺省时不参与序列化。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
}

impl EnumItem {
    /// 构造整数取值的枚举项。
    ///
    /// # 参数
    ///
    /// - `label`：展示文案。
    /// - `value`：整数判别值（对应库中的 `SMALLINT` 列）。
    /// - `color`：标签配色，取值见 [`color`]。
    ///
    /// # 返回值
    ///
    /// 不含补充说明的枚举项。
    pub fn int(label: &'static str, value: i16, color: &'static str) -> Self {
        Self {
            label,
            value: JsonValue::from(value),
            color,
            description: None,
        }
    }

    /// 构造字符串取值的枚举项。
    ///
    /// # 参数
    ///
    /// - `label`：展示文案。
    /// - `value`：字符串判别值（对应库中的 `VARCHAR` 列）。
    /// - `color`：标签配色，取值见 [`color`]。
    ///
    /// # 返回值
    ///
    /// 不含补充说明的枚举项。
    pub fn text(label: &'static str, value: &'static str, color: &'static str) -> Self {
        Self {
            label,
            value: JsonValue::from(value),
            color,
            description: None,
        }
    }

    /// 追加补充说明。
    ///
    /// # 参数
    ///
    /// - `description`：辅助文案。
    ///
    /// # 返回值
    ///
    /// 带补充说明的枚举项（链式调用）。
    pub fn with_description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
}

/// 一个模块的枚举表：枚举名 → 枚举项数组。
///
/// 用 [`BTreeMap`] 而非 `HashMap`：序列化后键按字典序稳定输出，便于接口快照对比
/// 与人工排查（无序输出会让「有没有变化」这个判断失效）。
pub type EnumsRes = BTreeMap<&'static str, Vec<EnumItem>>;

/// 组装枚举表。
///
/// 仅用于把「常量键 → 常量项数组」写成一行，避免每个模块重复
/// `BTreeMap::new()` + 多次 `insert`：
///
/// ```ignore
/// enums! {
///     "status" => status_items(),
///     "level" => level_items(),
/// }
/// ```
#[macro_export]
macro_rules! module_enums {
    ($($key:literal => $items:expr),* $(,)?) => {{
        let mut table = $crate::shared::enums::EnumsRes::new();
        $( table.insert($key, $items); )*
        table
    }};
}

/// 通用「启用 / 停用」两项（`status` 列，`1` 启用、`0` 停用）。
///
/// # 参数
///
/// - `enabled_label`：启用态的展示文案（系统域惯用「正常」，支撑域惯用「启用」）。
///
/// # 返回值
///
/// 启用（`success`）+ 停用（`danger`）两项，顺序为「启用在前」。
pub fn status_items(enabled_label: &'static str) -> Vec<EnumItem> {
    vec![
        EnumItem::int(enabled_label, 1, color::SUCCESS),
        EnumItem::int("停用", 0, color::DANGER),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_and_text_keep_native_json_type() {
        assert!(EnumItem::int("启用", 1, color::SUCCESS).value.is_i64());
        assert!(
            EnumItem::text("启用", "on", color::SUCCESS)
                .value
                .is_string()
        );
    }

    #[test]
    fn description_is_optional_and_chainable() {
        let plain = EnumItem::int("本人", 1, color::WARNING);
        assert!(plain.description.is_none());

        let described = EnumItem::int("本人", 1, color::WARNING).with_description("仅本人数据");
        assert_eq!(described.description, Some("仅本人数据"));
    }

    #[test]
    fn module_enums_keeps_all_keys() {
        let table = module_enums! {
            "status" => status_items("启用"),
            "level" => vec![EnumItem::int("普通", 1, color::DEFAULT)],
        };
        assert_eq!(table.len(), 2);
        assert_eq!(table["status"].len(), 2);
        assert_eq!(table["level"].len(), 1);
    }

    #[test]
    fn status_items_orders_enabled_first() {
        let items = status_items("正常");
        assert_eq!(items[0].label, "正常");
        assert_eq!(items[0].color, color::SUCCESS);
        assert_eq!(items[1].label, "停用");
        assert_eq!(items[1].color, color::DANGER);
    }

    #[test]
    fn enums_res_serializes_to_object_of_arrays() {
        let table = module_enums! { "status" => status_items("启用") };
        let json = serde_json::to_value(&table).expect("枚举表应可序列化");
        assert!(json.get("status").and_then(|v| v.as_array()).is_some());
        // 未设置 description 时不应出现在载荷里
        assert!(json["status"][0].get("description").is_none());
    }
}
