-- 系统用户表：后台账号及其登录安全状态
CREATE TABLE sys_user (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    username VARCHAR(50) NOT NULL, -- 登录账号（全局唯一）
    password VARCHAR(255) NOT NULL, -- 密码哈希（不存明文）
    real_name VARCHAR(50), -- 真实姓名
    email VARCHAR(100), -- 邮箱
    mobile VARCHAR(20), -- 手机号
    dept_id UUID, -- 所属部门 ID（sys_dept.id）
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    login_fail_count SMALLINT NOT NULL DEFAULT 0, -- 连续登录失败次数
    is_locked BOOLEAN NOT NULL DEFAULT FALSE, -- 是否锁定
    lock_time TIMESTAMPTZ, -- 锁定时间
    last_login_at TIMESTAMPTZ, -- 最近登录时间
    remark TEXT, -- 备注
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_user IS '系统用户表：后台账号及其登录安全状态';
COMMENT ON COLUMN sys_user.id IS '主键';
COMMENT ON COLUMN sys_user.username IS '登录账号（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_user.password IS '密码哈希（不存明文）';
COMMENT ON COLUMN sys_user.real_name IS '真实姓名';
COMMENT ON COLUMN sys_user.email IS '邮箱';
COMMENT ON COLUMN sys_user.mobile IS '手机号';
COMMENT ON COLUMN sys_user.dept_id IS '所属部门 ID（sys_dept.id，弱引用不建外键）';
COMMENT ON COLUMN sys_user.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_user.login_fail_count IS '连续登录失败次数，达到阈值后锁定账号';
COMMENT ON COLUMN sys_user.is_locked IS '是否锁定：true 已锁定 / false 未锁定';
COMMENT ON COLUMN sys_user.lock_time IS '锁定时间（未锁定为空）';
COMMENT ON COLUMN sys_user.last_login_at IS '最近登录时间（从未登录为空）';
COMMENT ON COLUMN sys_user.remark IS '备注';
COMMENT ON COLUMN sys_user.created_at IS '创建时间';
COMMENT ON COLUMN sys_user.updated_at IS '更新时间';
COMMENT ON COLUMN sys_user.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_user.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_user.deleted_at IS '软删时间（非空即已删除）';

-- 用户名唯一：仅约束未软删行，因此删除后可重建同名账号
CREATE UNIQUE INDEX uk_sys_user_username ON sys_user (username) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_user_dept_id ON sys_user (dept_id);
CREATE INDEX idx_sys_user_status ON sys_user (status);
