-- 审计日志表：记录关键变更的前后数据快照（只写不改删）
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    request_id VARCHAR(36) NOT NULL, -- 请求追踪 ID
    user_id UUID, -- 操作用户 ID
    username VARCHAR(50), -- 操作用户名（冗余，避免联表）
    operation VARCHAR(50) NOT NULL, -- 操作类型（LOGIN_SUCCESS / USER_CREATE / ROLE_ASSIGN ...）
    target_type VARCHAR(50), -- 操作对象类型（User / Role / Permission ...）
    target_id VARCHAR(100), -- 操作对象 ID
    before_data JSONB, -- 变更前数据快照
    after_data JSONB, -- 变更后数据快照
    detail JSONB, -- 补充详情（如登录 IP、设备信息）
    ip INET, -- 客户端 IP
    user_agent TEXT, -- 客户端 User-Agent
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE audit_logs IS '审计日志表：记录关键变更的前后数据快照（只写不改删）';
COMMENT ON COLUMN audit_logs.id IS '主键';
COMMENT ON COLUMN audit_logs.request_id IS '请求追踪 ID';
COMMENT ON COLUMN audit_logs.user_id IS '操作用户 ID';
COMMENT ON COLUMN audit_logs.username IS '操作用户名（冗余，避免联表）';
COMMENT ON COLUMN audit_logs.operation IS '操作类型（LOGIN_SUCCESS / USER_CREATE / ROLE_ASSIGN ...）';
COMMENT ON COLUMN audit_logs.target_type IS '操作对象类型（User / Role / Permission ...）';
COMMENT ON COLUMN audit_logs.target_id IS '操作对象 ID';
COMMENT ON COLUMN audit_logs.before_data IS '变更前数据快照（JSON）';
COMMENT ON COLUMN audit_logs.after_data IS '变更后数据快照（JSON）';
COMMENT ON COLUMN audit_logs.detail IS '补充详情（如登录 IP、设备信息）';
COMMENT ON COLUMN audit_logs.ip IS '客户端 IP';
COMMENT ON COLUMN audit_logs.user_agent IS '客户端 User-Agent';
COMMENT ON COLUMN audit_logs.created_at IS '创建时间（操作发生时刻）';

CREATE INDEX idx_audit_user_id ON audit_logs (user_id);
CREATE INDEX idx_audit_operation ON audit_logs (operation);
CREATE INDEX idx_audit_created_at ON audit_logs (created_at DESC);
