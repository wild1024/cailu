//! 日志插件：把日志总线的生命周期挂到应用插件体系上。
//!
//! 插件化带来两处收益：
//!
//! - 随应用 `start()` 一并拉起后台 worker，早于 HTTP 监听开始接收请求；
//! - 进程优雅退出时 `stop()` 会等待已入队日志全部落库，避免尾部日志丢失。
//!
//! # IP 归属地开关的运行期同步
//!
//! [`crate::support::ipgeo`] 把开关快照在进程级原子量里（读取侧在日志 worker 上
//! 逐条调用，必须是零 IO 的同步读）。此前只在启动时读一次
//! `sys_config` 的 [`constants::CONFIG_KEY_IP_LOCATION`]，管理端在运行期改这个参数
//! 不生效，只能重启。现在改为向参数快照**注册监听器**：参数写操作或
//! `POST /support/config/reload` 触发重载后，回调立即把新值推给原子量。

use std::sync::Arc;

use alun::prelude::*;
use async_trait::async_trait;

use crate::shared::constants;
use crate::support::config::model;

/// 日志总线插件。
pub struct LogPlugin;

impl LogPlugin {
    /// 创建日志插件。
    ///
    /// # 返回值
    ///
    /// 无状态的日志插件实例。
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogPlugin {
    /// 等价于 [`LogPlugin::new`]。
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for LogPlugin {
    /// 插件名称（用于插件管理与日志）。
    fn name(&self) -> &str {
        "x-log-bus"
    }

    /// 启动日志总线：初始化通道与 worker，并订阅 IP 归属地开关。
    ///
    /// # 返回值
    ///
    /// 恒返回 `Ok(())`（日志属旁路能力，初始化异常不阻断服务启动）。
    async fn start(&self) -> std::result::Result<(), Error> {
        crate::support::log::bus::init();
        // 订阅参数快照：注册时会以当前快照回调一次，之后每次参数变更由
        // support::config 广播刷新，无需重启
        model::subscribe_runtime(
            "log.ip_location",
            Arc::new(|snapshot| {
                // 缺省启用，仅在显式配置为 false / 0 时关闭
                let enabled = snapshot
                    .get(constants::CONFIG_KEY_IP_LOCATION)
                    .and_then(|raw| model::parse_bool(raw))
                    .unwrap_or(true);
                crate::support::ipgeo::set_enabled(enabled);
            }),
        );
        Ok(())
    }

    /// 停止日志总线：等待已入队日志全部落库。
    ///
    /// # 返回值
    ///
    /// 恒返回 `Ok(())`。
    async fn stop(&self) -> std::result::Result<(), Error> {
        crate::support::log::bus::flush().await;
        Ok(())
    }
}