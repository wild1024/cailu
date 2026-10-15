-- 定时任务表：cron 调度的任务定义
CREATE TABLE sys_job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    job_name VARCHAR(100) NOT NULL, -- 任务名称（全局唯一）
    job_group VARCHAR(50) NOT NULL DEFAULT 'DEFAULT', -- 任务分组
    cron_expr VARCHAR(100) NOT NULL, -- cron 表达式
    invoke_target VARCHAR(255) NOT NULL, -- 调用目标（注册的函数 / 处理器名）
    params JSONB, -- 任务参数
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    last_run_at TIMESTAMPTZ, -- 上次执行时间
    next_run_at TIMESTAMPTZ, -- 下次执行时间
    remark TEXT, -- 备注
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_job IS '定时任务表：cron 调度的任务定义';
COMMENT ON COLUMN sys_job.id IS '主键';
COMMENT ON COLUMN sys_job.job_name IS '任务名称（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_job.job_group IS '任务分组';
COMMENT ON COLUMN sys_job.cron_expr IS 'cron 表达式';
COMMENT ON COLUMN sys_job.invoke_target IS '调用目标（注册的函数 / 处理器名）';
COMMENT ON COLUMN sys_job.params IS '任务参数（JSON）';
COMMENT ON COLUMN sys_job.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_job.last_run_at IS '上次执行时间';
COMMENT ON COLUMN sys_job.next_run_at IS '下次执行时间';
COMMENT ON COLUMN sys_job.remark IS '备注';
COMMENT ON COLUMN sys_job.created_at IS '创建时间';
COMMENT ON COLUMN sys_job.updated_at IS '更新时间';
COMMENT ON COLUMN sys_job.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_job.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_job.deleted_at IS '软删时间（非空即已删除）';

-- 任务名称唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_job_name ON sys_job (job_name) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_job_status ON sys_job (status);
