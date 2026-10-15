//! 健康检查接口。
//!
//! 供负载均衡、容器编排（就绪/存活探针）与发布验证脚本调用。该接口不鉴权、不查库，
//! 只反映「HTTP 服务进程是否存活」，避免下游依赖抖动导致探针误判并触发无谓重启。

use alun::prelude::*;

/// 健康检查响应体。
#[derive(serde::Serialize)]
pub struct HealthRes {
    /// 服务状态，固定为 `ok`。
    pub status: String,
    /// 应用名称，取自配置 `app_name`。
    pub app: String,
    /// 当前生效的配置 profile。
    pub profile: String,
    /// 服务端当前时间（RFC 3339）。
    pub time: String,
}

/// 存活探针：返回服务状态与运行环境信息。
///
/// # 装配
///
/// 本接口需要在**全局前缀之外**（根路径 `/health`）暴露，因此不使用
/// `#[alun::get]` 宏（宏声明的路由会被 `[router] prefix` 嵌套），
/// 改由 [`crate::run`] 通过 `App::get("/health", ...)` 手动注册。
///
/// 免认证由 `config.toml` 的 `[middleware.auth].ignore_paths` 中的 `"!/health"`
/// 承担（`!` 表示绝对路径，不参与前缀拼接）。
///
/// # 返回值
///
/// 始终返回 `code = 0` 的统一响应体，`data` 为 [`HealthRes`]。
pub async fn health() -> Res<HealthRes> {
    Res::ok(HealthRes {
        status: "ok".to_string(),
        app: cfg().app_name.clone(),
        profile: cfg().profile.clone(),
        time: chrono::Utc::now().to_rfc3339(),
    })
}
