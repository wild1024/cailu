-- 密码变更日志表：记录重置 / 修改 / 强制改密的操作者与结果（只写不改删）
CREATE TABLE password_change_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    request_id VARCHAR(36) NOT NULL, -- 请求追踪 ID
    user_id UUID NOT NULL, -- 被修改密码的用户 ID
    username VARCHAR(50) NOT NULL, -- 被修改密码的用户名
    change_type VARCHAR(20) NOT NULL, -- 变更类型：RESET / UPDATE / FORCED
    operator_type VARCHAR(20) NOT NULL, -- 操作者类型：SELF / ADMIN / SYSTEM
    operator_id UUID, -- 操作者 ID（本人修改时等于 user_id）
    ip INET, -- 客户端 IP
    user_agent TEXT, -- 客户端 User-Agent
    change_status VARCHAR(20) NOT NULL DEFAULT 'SUCCESS', -- 变更状态：SUCCESS / FAIL
    error_message VARCHAR(255), -- 失败原因
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE password_change_logs IS '密码变更日志表：记录重置 / 修改 / 强制改密的操作者与结果（只写不改删）';
COMMENT ON COLUMN password_change_logs.id IS '主键';
COMMENT ON COLUMN password_change_logs.request_id IS '请求追踪 ID';
COMMENT ON COLUMN password_change_logs.user_id IS '被修改密码的用户 ID';
COMMENT ON COLUMN password_change_logs.username IS '被修改密码的用户名';
COMMENT ON COLUMN password_change_logs.change_type IS '变更类型：RESET / UPDATE / FORCED';
COMMENT ON COLUMN password_change_logs.operator_type IS '操作者类型：SELF / ADMIN / SYSTEM';
COMMENT ON COLUMN password_change_logs.operator_id IS '操作者 ID（本人修改时等于 user_id）';
COMMENT ON COLUMN password_change_logs.ip IS '客户端 IP';
COMMENT ON COLUMN password_change_logs.user_agent IS '客户端 User-Agent';
COMMENT ON COLUMN password_change_logs.change_status IS '变更状态：SUCCESS / FAIL';
COMMENT ON COLUMN password_change_logs.error_message IS '失败原因';
COMMENT ON COLUMN password_change_logs.created_at IS '创建时间（操作发生时刻）';

CREATE INDEX idx_pwd_user_id ON password_change_logs (user_id);
CREATE INDEX idx_pwd_username ON password_change_logs (username);
CREATE INDEX idx_pwd_created_at ON password_change_logs (created_at DESC);
