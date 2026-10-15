//! 支撑功能模块（业务容器）。
//!
//! 承载「与具体业务域无关、但被各业务模块共用」的支撑能力。每个子模块统一遵循
//! `mod.rs + handler.rs + dto.rs + model.rs` 结构。
//!
//! | 子模块 | 职责 | 交付阶段 |
//! |--------|------|----------|
//! | [`health`] | 存活探针 `GET /health`，供负载均衡与容器健康检查使用 | P0 |
//! | [`dict`] | 数据字典（字典类型 / 字典项） | P3 |
//! | [`config`] | 系统参数（键值配置，支持按环境覆盖） | P3 |
//! | [`dashboard`] | 工作台概览（首页指标卡片与登录趋势聚合） | P5 |
//! | [`file`] | 文件上传、下载、元数据管理 | P3 |
//! | [`log`] | 操作日志查询与清理 | P3 |
//! | [`job`] | 定时任务定义与执行日志 | P4 |
//! | [`online`] | 在线用户会话（登录令牌维度的在线列表 / 强制下线） | P4 |
//! | [`ipgeo`] | IP 归属地解析（供日志 worker 补齐 `auth_logs.location`） | P4 |
//!
//! P1 阶段交付各子模块的强类型模型（`model.rs`），接口层（`handler.rs` /
//! `dto.rs`）随 P3 / P4 补齐。

pub mod config;
pub mod dashboard;
pub mod dict;
pub mod file;
pub mod health;
pub(crate) mod ipgeo;
pub mod job;
pub mod log;
pub mod online;
