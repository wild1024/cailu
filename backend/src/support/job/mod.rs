//! 定时任务子模块：任务定义、执行日志与管理接口。
//!
//! | 文件 | 内容 |
//! |------|------|
//! | [`model`] | [`model::SysJobModel`]（`sys_job`）、[`model::SysJobLogModel`]（`sys_job_log`）、[`model::JobLogStatus`] |
//! | [`dto`] | 请求 / 响应结构（[`dto::JobRes`] / [`dto::JobLogRes`] 等） |
//! | [`handler`] | 定时任务管理接口 |
//!
//! 对外路径前缀 `/api/support/job`。调度基于框架 `SchedulerPlugin`（cron），
//! 任务定义存 `sys_job`，每次执行写 `sys_job_log` 并回写
//! `last_run_at` / `next_run_at`。新建 / 更新 / 启停 / 删除后经
//! [`crate::task::scheduler`] 的 `reload()` 重新装载调度。
//!
//! # 接口清单
//!
//! | 方法 | 路径 | 权限码 | 说明 |
//! |------|------|--------|------|
//! | GET | `/api/support/job/page` | `support:job:list` | 分页查询任务（`page` / `page_size` / `job_name?` / `job_group?` / `status?`） |
//! | GET | `/api/support/job/{id}` | `support:job:list` | 任务详情 |
//! | POST | `/api/support/job` | `support:job:create` | 新建任务 |
//! | PUT | `/api/support/job/{id}` | `support:job:update` | 更新任务 |
//! | DELETE | `/api/support/job/{id}` | `support:job:delete` | 删除任务 |
//! | PATCH | `/api/support/job/{id}/status` | `support:job:update` | 启停任务（body `{ "status": 0 \| 1 }`） |
//! | POST | `/api/support/job/{id}/trigger` | `support:job:trigger` | 立即执行一次（同步等待执行完成） |
//! | GET | `/api/support/job/log/page` | `support:job:list` | 分页查询执行日志（`page` / `page_size` / `job_id?` / `job_name?` / `status?`） |
//! | GET | `/api/support/job/log/{id}` | `support:job:list` | 执行日志详情 |

pub mod dto;
pub mod handler;
pub mod model;
