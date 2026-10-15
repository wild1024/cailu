-- 定时任务日志表：每次任务执行的明细（不建外键，任务删除后日志保留）
CREATE TABLE sys_job_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    job_id UUID NOT NULL, -- 任务 ID（sys_job.id，无外键约束）
    job_name VARCHAR(100) NOT NULL, -- 任务名称（冗余）
    invoke_target VARCHAR(255) NOT NULL, -- 调用目标（冗余）
    params JSONB, -- 本次执行参数
    status SMALLINT NOT NULL, -- 执行状态：1 成功 / 0 失败
    message TEXT, -- 执行结果消息
    error TEXT, -- 异常信息（成功时为空）
    started_at TIMESTAMPTZ NOT NULL, -- 开始时间
    finished_at TIMESTAMPTZ, -- 结束时间
    duration_ms INT, -- 耗时（毫秒）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE sys_job_log IS '定时任务日志表：每次任务执行的明细（不建外键，任务删除后日志保留）';
COMMENT ON COLUMN sys_job_log.id IS '主键';
COMMENT ON COLUMN sys_job_log.job_id IS '任务 ID（sys_job.id，无外键约束）';
COMMENT ON COLUMN sys_job_log.job_name IS '任务名称（冗余）';
COMMENT ON COLUMN sys_job_log.invoke_target IS '调用目标（冗余）';
COMMENT ON COLUMN sys_job_log.params IS '本次执行参数（JSON）';
COMMENT ON COLUMN sys_job_log.status IS '执行状态：1 成功 / 0 失败';
COMMENT ON COLUMN sys_job_log.message IS '执行结果消息';
COMMENT ON COLUMN sys_job_log.error IS '异常信息（成功时为空）';
COMMENT ON COLUMN sys_job_log.started_at IS '开始时间';
COMMENT ON COLUMN sys_job_log.finished_at IS '结束时间';
COMMENT ON COLUMN sys_job_log.duration_ms IS '耗时（毫秒）';
COMMENT ON COLUMN sys_job_log.created_at IS '创建时间';

CREATE INDEX idx_sys_job_log_job_id ON sys_job_log (job_id);
CREATE INDEX idx_sys_job_log_created_at ON sys_job_log (created_at DESC);
