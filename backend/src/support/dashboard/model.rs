//! 工作台概览的数据访问。
//!
//! 数据全部来自既有的日志与业务表（`auth_logs` / `sys_user` / `sys_job`），**不新增统计表**：
//!
//! - 指标卡片：一条聚合 SQL 取回 7 个标量，缓存中的在线会话数由调用方（handler）补入；
//! - 两张趋势图：共用「近 N 天日期序列 `generate_series` 左连接日志」的同一形状 SQL，
//!   缺数据的自然日以 0 / `NULL` 占位，保证 X 轴连续。
//!
//! 日期口径统一按 [`REPORT_TIME_ZONE`] 切自然日：数据库所在机器的时区通常不是国内时区，
//! 直接用 `now()` 会让「今日登录」与用户看到的日期错开 8 小时。

use alun::prelude::*;

use crate::support::dashboard::dto::{MetricCardRes, TrendLineRes, TrendRes};

/// 统计口径使用的时区（自然日切分基准）。
const REPORT_TIME_ZONE: &str = "Asia/Shanghai";

/// 趋势窗口天数（含当天），同时下发到响应供前端展示。
pub const TREND_DAYS: i64 = 14;

/// 指标聚合结果（标量，字段名与 SQL 别名一致）。
#[derive(Debug, Clone, Copy)]
pub struct MetricRow {
    /// 用户总数（未软删）。
    pub user_total: i64,
    /// 今日成功登录的去重人数。
    pub active_today: i64,
    /// 今日登录尝试次数（成功 + 失败）。
    pub login_today: i64,
    /// 今日登录失败率（百分比整数，分母为 0 时取 0）。
    pub fail_rate: i64,
    /// 近 7 天成功登录的平均耗时（毫秒，无样本时取 0）。
    pub avg_login_ms: i64,
    /// 定时任务总数。
    pub job_total: i64,
    /// 状态为启用的定时任务数。
    pub job_enabled: i64,
    /// 角色总数（未软删）。
    pub role_total: i64,
    /// 菜单总数（未软删）。
    pub menu_total: i64,
    /// 部门总数（未软删）。
    pub dept_total: i64,
    /// 字典类型总数。
    pub dict_type_total: i64,
    /// 近 7 天操作日志条数。
    pub op_log_week: i64,
}

/// 近 N 天的登录次数趋势行。
#[derive(Debug, Clone)]
pub struct LoginTrendRow {
    /// 日期标签 `MM-DD`。
    pub label: String,
    /// 成功次数。
    pub success: i64,
    /// 失败次数。
    pub fail: i64,
}

/// 近 N 天的登录耗时趋势行。
#[derive(Debug, Clone)]
pub struct LatencyTrendRow {
    /// 日期标签 `MM-DD`。
    pub label: String,
    /// 当日成功登录的平均耗时（毫秒，无样本时为空）。
    pub avg_ms: Option<i64>,
}

/// 指标查询：一次往返取回全部标量。
///
/// # 返回值
///
/// 成功返回 [`MetricRow`]；无数据（如空库）时各数值为 0。
///
/// # 异常情况
///
/// 数据库异常时返回 [`DbError`]，由调用方包装成接口错误。
pub async fn load_metrics() -> std::result::Result<MetricRow, DbError> {
    let sql = format!(
        r#"
WITH scope AS (
    SELECT
        (date_trunc('day', now() AT TIME ZONE '{}'))::timestamptz               AS day_start,
        ((date_trunc('day', now() AT TIME ZONE '{}')) + interval '1 day')::timestamptz
                                                                                AS day_end,
        ((date_trunc('day', now() AT TIME ZONE '{}')) - interval '6 days')::timestamptz
                                                                                AS week_start
)
SELECT
    (SELECT COUNT(*) FROM sys_user WHERE deleted_at IS NULL)                    AS user_total,
    (SELECT COUNT(DISTINCT a.user_id)
       FROM auth_logs a, scope
      WHERE a.auth_type = 'LOGIN' AND a.auth_status = 'SUCCESS'
        AND a.created_at >= scope.day_start AND a.created_at < scope.day_end)  AS active_today,
    (SELECT COUNT(*)
       FROM auth_logs a, scope
      WHERE a.auth_type = 'LOGIN'
        AND a.created_at >= scope.day_start AND a.created_at < scope.day_end)  AS login_today,
    (SELECT COALESCE(ROUND(100.0 * COUNT(*) FILTER (WHERE a.auth_status = 'FAIL')
                           / NULLIF(COUNT(*), 0))::bigint, 0)
       FROM auth_logs a, scope
      WHERE a.auth_type = 'LOGIN'
        AND a.created_at >= scope.day_start AND a.created_at < scope.day_end)  AS fail_rate,
    (SELECT COALESCE(ROUND(AVG(a.login_duration))::bigint, 0)
       FROM auth_logs a, scope
      WHERE a.auth_type = 'LOGIN' AND a.auth_status = 'SUCCESS'
        AND a.login_duration IS NOT NULL
        AND a.created_at >= scope.week_start AND a.created_at < scope.day_end) AS avg_login_ms,
    (SELECT COUNT(*) FROM sys_job WHERE deleted_at IS NULL)                     AS job_total,
    (SELECT COUNT(*) FROM sys_job WHERE deleted_at IS NULL AND status = 1)      AS job_enabled,
    (SELECT COUNT(*) FROM sys_role WHERE deleted_at IS NULL)                    AS role_total,
    (SELECT COUNT(*) FROM sys_menu WHERE deleted_at IS NULL)                    AS menu_total,
    (SELECT COUNT(*) FROM sys_dept WHERE deleted_at IS NULL)                    AS dept_total,
    (SELECT COUNT(*) FROM sys_dict_type)                                        AS dict_type_total,
    (SELECT COUNT(*)
       FROM operation_logs o, scope
      WHERE o.created_at >= scope.week_start AND o.created_at < scope.day_end)  AS op_log_week
"#,
        REPORT_TIME_ZONE,
        REPORT_TIME_ZONE,
        REPORT_TIME_ZONE,
    );

    let row = db()
        .find_first(&sql)
        .await?
        .ok_or(DbError::NotFound)?;

    Ok(MetricRow {
        user_total: row.get_typed::<i64>("user_total").unwrap_or_default(),
        active_today: row.get_typed::<i64>("active_today").unwrap_or_default(),
        login_today: row.get_typed::<i64>("login_today").unwrap_or_default(),
        fail_rate: row.get_typed::<i64>("fail_rate").unwrap_or_default(),
        avg_login_ms: row.get_typed::<i64>("avg_login_ms").unwrap_or_default(),
        job_total: row.get_typed::<i64>("job_total").unwrap_or_default(),
        job_enabled: row.get_typed::<i64>("job_enabled").unwrap_or_default(),
        role_total: row.get_typed::<i64>("role_total").unwrap_or_default(),
        menu_total: row.get_typed::<i64>("menu_total").unwrap_or_default(),
        dept_total: row.get_typed::<i64>("dept_total").unwrap_or_default(),
        dict_type_total: row.get_typed::<i64>("dict_type_total").unwrap_or_default(),
        op_log_week: row.get_typed::<i64>("op_log_week").unwrap_or_default(),
    })
}

/// 登录次数趋势：近 [`TREND_DAYS`] 天按天拆分成功 / 失败。
///
/// # 异常情况
///
/// 数据库异常时返回 [`DbError`]。
pub async fn load_login_trend() -> std::result::Result<Vec<LoginTrendRow>, DbError> {
    trend_rows(format!(
        r#"
WITH days AS (
    SELECT generate_series(
        (date_trunc('day', now() AT TIME ZONE '{}') - make_interval(days => $1::int))::timestamptz,
        (date_trunc('day', now() AT TIME ZONE '{}'))::timestamptz,
        interval '1 day'
    )::date AS d
)
SELECT to_char(days.d, 'MM-DD')                        AS label,
       COUNT(a.id) FILTER (WHERE a.auth_status = 'SUCCESS')::bigint AS success,
       COUNT(a.id) FILTER (WHERE a.auth_status = 'FAIL')::bigint    AS fail
  FROM days
  LEFT JOIN auth_logs a
         ON a.auth_type = 'LOGIN'
        AND a.created_at >= days.d
        AND a.created_at < days.d + interval '1 day'
 GROUP BY days.d
 ORDER BY days.d
"#,
        REPORT_TIME_ZONE,
        REPORT_TIME_ZONE,
    ))
    .await
}

/// 登录耗时趋势：近 [`TREND_DAYS`] 天按天统计成功登录的平均耗时。
///
/// # 异常情况
///
/// 数据库异常时返回 [`DbError`]。
pub async fn load_latency_trend() -> std::result::Result<Vec<LatencyTrendRow>, DbError> {
    trend_rows(format!(
        r#"
WITH days AS (
    SELECT generate_series(
        (date_trunc('day', now() AT TIME ZONE '{}') - make_interval(days => $1::int))::timestamptz,
        (date_trunc('day', now() AT TIME ZONE '{}'))::timestamptz,
        interval '1 day'
    )::date AS d
)
SELECT to_char(days.d, 'MM-DD')                        AS label,
       ROUND(AVG(a.login_duration))::bigint            AS avg_ms
  FROM days
  LEFT JOIN auth_logs a
         ON a.auth_type = 'LOGIN' AND a.auth_status = 'SUCCESS'
        AND a.login_duration IS NOT NULL
        AND a.created_at >= days.d
        AND a.created_at < days.d + interval '1 day'
 GROUP BY days.d
 ORDER BY days.d
"#,
        REPORT_TIME_ZONE,
        REPORT_TIME_ZONE,
    ))
    .await
}

/// 把「公共辅助 + 具体 SELECT」拼接后的通用趋势查询。
///
/// # 参数
///
/// - `sql`：已拼好 `WITH days AS (...)` 的完整 SELECT。
///
/// # 返回值
///
/// 按日期升序的趋势行；`avg_ms` 列缺失 / 为 `NULL` 时解析为 `None`。
///
/// # 异常情况
///
/// 数据库异常或缺少预期列时返回 [`DbError`]。
async fn trend_rows<T: for<'r> FromTrendRow<'r>>(sql: String) -> std::result::Result<Vec<T>, DbError> {
    let params = vec![Value::from(TREND_DAYS)];
    let rows = db().find_with_params(&sql, params).await?;
    rows.iter().map(T::from_row).collect()
}

/// 趋势行解码：把一行结果集转成具体趋势类型。
trait FromTrendRow<'r> {
    /// 从结果行解码。
    ///
    /// # 异常情况
    ///
    /// 缺少必要列时返回 [`DbError`]。
    fn from_row(row: &'r Row) -> std::result::Result<Self, DbError>
    where
        Self: Sized;
}

impl<'r> FromTrendRow<'r> for LoginTrendRow {
    fn from_row(row: &'r Row) -> std::result::Result<Self, DbError> {
        Ok(LoginTrendRow {
            label: row.get_typed::<String>("label").unwrap_or_default(),
            success: row.get_typed::<i64>("success").unwrap_or_default(),
            fail: row.get_typed::<i64>("fail").unwrap_or_default(),
        })
    }
}

impl<'r> FromTrendRow<'r> for LatencyTrendRow {
    fn from_row(row: &'r Row) -> std::result::Result<Self, DbError> {
        Ok(LatencyTrendRow {
            label: row.get_typed::<String>("label").unwrap_or_default(),
            avg_ms: row.get_typed::<i64>("avg_ms").ok(),
        })
    }
}

/// 指标卡片装配：一行数据 → 前端可直接渲染的卡片列表。
///
/// # 参数
///
/// - `metrics`：标量聚合结果。
/// - `online_users`：在线会话数（来自缓存，不参与 SQL 聚合）。
///
/// # 返回值
///
/// 固定顺序的 12 张卡片（前端按 xl 6 列 / md 4 列铺开，正好两行），口径与顺序保持稳定，
/// 避免前端缓存的旧布局在不同后端版本间跳动。
pub fn build_metric_cards(metrics: &MetricRow, online_users: usize) -> Vec<MetricCardRes> {
    vec![
        MetricCardRes {
            key: "user_total".to_string(),
            label: "用户总数".to_string(),
            value: metrics.user_total,
            extra: 0,
            unit: String::new(),
            icon: "user".to_string(),
            theme: "blue".to_string(),
            tip: "系统内未删除的账号数".to_string(),
        },
        MetricCardRes {
            key: "online_user".to_string(),
            label: "在线用户".to_string(),
            value: online_users as i64,
            extra: 0,
            unit: String::new(),
            icon: "usergroup".to_string(),
            theme: "green".to_string(),
            tip: "令牌未过期的会话数".to_string(),
        },
        MetricCardRes {
            key: "active_today".to_string(),
            label: "今日活跃".to_string(),
            value: metrics.active_today,
            extra: 0,
            unit: String::new(),
            icon: "user-check".to_string(),
            theme: "cyan".to_string(),
            tip: "今日成功登录的去重人数".to_string(),
        },
        MetricCardRes {
            key: "login_today".to_string(),
            label: "今日登录".to_string(),
            value: metrics.login_today,
            extra: metrics.fail_rate,
            unit: String::new(),
            icon: "login".to_string(),
            theme: "purple".to_string(),
            tip: "登录尝试次数（失败占比见附加数）".to_string(),
        },
        MetricCardRes {
            key: "fail_rate".to_string(),
            label: "登录失败率".to_string(),
            value: metrics.fail_rate,
            extra: metrics.login_today,
            unit: "%".to_string(),
            icon: "time".to_string(),
            theme: "orange".to_string(),
            tip: "今日失败次数占登录尝试的比例".to_string(),
        },
        MetricCardRes {
            key: "avg_login_ms".to_string(),
            label: "平均登录耗时".to_string(),
            value: metrics.avg_login_ms,
            extra: 0,
            unit: "ms".to_string(),
            icon: "speed".to_string(),
            theme: "red".to_string(),
            tip: "近 7 天成功登录的平均耗时".to_string(),
        },
        MetricCardRes {
            key: "job_total".to_string(),
            label: "定时任务".to_string(),
            value: metrics.job_total,
            extra: metrics.job_enabled,
            unit: String::new(),
            icon: "time".to_string(),
            theme: "blue".to_string(),
            tip: "任务总数（启用数见附加数）".to_string(),
        },
        MetricCardRes {
            key: "role_total".to_string(),
            label: "角色总数".to_string(),
            value: metrics.role_total,
            extra: 0,
            unit: String::new(),
            icon: "usergroup".to_string(),
            theme: "purple".to_string(),
            tip: "未删除的角色数（含超管）".to_string(),
        },
        MetricCardRes {
            key: "menu_total".to_string(),
            label: "菜单总数".to_string(),
            value: metrics.menu_total,
            extra: 0,
            unit: String::new(),
            icon: "menu".to_string(),
            theme: "cyan".to_string(),
            tip: "未删除的菜单 + 按钮权限点".to_string(),
        },
        MetricCardRes {
            key: "dept_total".to_string(),
            label: "部门总数".to_string(),
            value: metrics.dept_total,
            extra: 0,
            unit: String::new(),
            icon: "share".to_string(),
            theme: "green".to_string(),
            tip: "组织树中未删除的部门数".to_string(),
        },
        MetricCardRes {
            key: "dict_type_total".to_string(),
            label: "字典类型".to_string(),
            value: metrics.dict_type_total,
            extra: 0,
            unit: String::new(),
            icon: "book".to_string(),
            theme: "orange".to_string(),
            tip: "字典分类数量（字典项不计入）".to_string(),
        },
        MetricCardRes {
            key: "op_log_week".to_string(),
            label: "近 7 天操作日志".to_string(),
            value: metrics.op_log_week,
            extra: 0,
            unit: "条".to_string(),
            icon: "file-copy".to_string(),
            theme: "red".to_string(),
            tip: "近 7 天写入的操作审计条数".to_string(),
        },
    ]
}

/// 登录次数趋势 → 前端趋势结构。
///
/// # 参数
///
/// - `rows`：按日期升序的趋势行。
///
/// # 返回值
///
/// X 轴日期 + 「成功 / 失败」两条折线。
pub fn to_login_trend_res(rows: &[LoginTrendRow]) -> TrendRes {
    TrendRes {
        dates: rows.iter().map(|r| r.label.clone()).collect(),
        lines: vec![
            TrendLineRes {
                name: "成功".to_string(),
                data: rows.iter().map(|r| Some(r.success)).collect(),
            },
            TrendLineRes {
                name: "失败".to_string(),
                data: rows.iter().map(|r| Some(r.fail)).collect(),
            },
        ],
    }
}

/// 登录耗时趋势 → 前端趋势结构。
///
/// # 参数
///
/// - `rows`：按日期升序的趋势行。
///
/// # 返回值
///
/// X 轴日期 + 「平均耗时」一条折线（无样本日为 `null`）。
pub fn to_latency_trend_res(rows: &[LatencyTrendRow]) -> TrendRes {
    TrendRes {
        dates: rows.iter().map(|r| r.label.clone()).collect(),
        lines: vec![TrendLineRes {
            name: "平均耗时(ms)".to_string(),
            data: rows.iter().map(|r| r.avg_ms).collect(),
        }],
    }
}
