-- 安全日志表：登录失败、越权、可疑注入等安全事件（只写不改删）
CREATE TABLE security_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    request_id VARCHAR(36), -- 请求追踪 ID（非请求触发时为空）
    user_id UUID, -- 关联用户 ID（可能为空）
    username VARCHAR(50), -- 用户名（尝试登录或操作的用户名）
    event_type VARCHAR(50) NOT NULL, -- 事件类型：LOGIN_FAIL / PERMISSION_DENY / SQL_INJECT_ATTEMPT ...
    detail JSONB, -- 事件详情（脱敏）
    ip INET, -- 客户端 IP
    user_agent TEXT, -- 客户端 User-Agent
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE security_logs IS '安全日志表：登录失败、越权、可疑注入等安全事件（只写不改删）';
COMMENT ON COLUMN security_logs.id IS '主键';
COMMENT ON COLUMN security_logs.request_id IS '请求追踪 ID（非请求触发时为空）';
COMMENT ON COLUMN security_logs.user_id IS '关联用户 ID（可能为空）';
COMMENT ON COLUMN security_logs.username IS '用户名（尝试登录或操作的用户名）';
COMMENT ON COLUMN security_logs.event_type IS '事件类型：LOGIN_FAIL / PERMISSION_DENY / SQL_INJECT_ATTEMPT ...';
COMMENT ON COLUMN security_logs.detail IS '事件详情（脱敏 JSON）';
COMMENT ON COLUMN security_logs.ip IS '客户端 IP';
COMMENT ON COLUMN security_logs.user_agent IS '客户端 User-Agent';
COMMENT ON COLUMN security_logs.created_at IS '创建时间（事件发生时刻）';

CREATE INDEX idx_sec_user_id ON security_logs (user_id);
CREATE INDEX idx_sec_event_type ON security_logs (event_type);
CREATE INDEX idx_sec_created_at ON security_logs (created_at DESC);
