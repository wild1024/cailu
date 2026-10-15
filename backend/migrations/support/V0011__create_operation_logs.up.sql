-- 操作日志表：记录接口调用的操作人、目标与耗时（配合中间件写入）
CREATE TABLE operation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    request_id VARCHAR(36) NOT NULL, -- 请求追踪 ID（贯穿一次请求）
    user_id UUID, -- 操作人 ID（匿名请求为空）
    username VARCHAR(50), -- 操作人账号（冗余，便于直接展示）
    operation VARCHAR(100) NOT NULL, -- 操作描述
    target_type VARCHAR(50), -- 操作对象类型（如 sys_user）
    target_id VARCHAR(100), -- 操作对象 ID
    detail JSONB, -- 操作详情（变更前后等结构化数据）
    ip INET, -- 客户端 IP
    user_agent TEXT, -- 客户端 User-Agent
    duration_ms INT, -- 处理耗时（毫秒）
    request_path VARCHAR(255), -- 请求路径
    request_method VARCHAR(10), -- 请求方法（GET / POST ...）
    request_params JSONB, -- 请求参数（脱敏后）
    response_data JSONB, -- 响应数据（脱敏后的结构化快照）
    status_code INT, -- 响应状态码
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE operation_logs IS '操作日志表：记录接口调用的操作人、目标与耗时（只写不改，按时间清理）';
COMMENT ON COLUMN operation_logs.id IS '主键';
COMMENT ON COLUMN operation_logs.request_id IS '请求追踪 ID（贯穿一次请求，跨服务关联）';
COMMENT ON COLUMN operation_logs.user_id IS '操作人 ID（匿名请求为空）';
COMMENT ON COLUMN operation_logs.username IS '操作人账号（冗余，便于直接展示）';
COMMENT ON COLUMN operation_logs.operation IS '操作描述（如 USER_CREATE）';
COMMENT ON COLUMN operation_logs.target_type IS '操作对象类型（如 sys_user）';
COMMENT ON COLUMN operation_logs.target_id IS '操作对象 ID';
COMMENT ON COLUMN operation_logs.detail IS '操作详情（变更前后等结构化数据，已脱敏）';
COMMENT ON COLUMN operation_logs.ip IS '客户端 IP';
COMMENT ON COLUMN operation_logs.user_agent IS '客户端 User-Agent';
COMMENT ON COLUMN operation_logs.duration_ms IS '处理耗时（毫秒）';
COMMENT ON COLUMN operation_logs.request_path IS '请求路径';
COMMENT ON COLUMN operation_logs.request_method IS '请求方法（GET / POST ...）';
COMMENT ON COLUMN operation_logs.request_params IS '请求参数（脱敏后）';
COMMENT ON COLUMN operation_logs.response_data IS '响应数据（脱敏后的结构化快照）';
COMMENT ON COLUMN operation_logs.status_code IS '响应状态码';
COMMENT ON COLUMN operation_logs.created_at IS '创建时间（操作发生时刻）';

CREATE INDEX idx_op_user_id ON operation_logs (user_id);
CREATE INDEX idx_op_created_at ON operation_logs (created_at DESC);
CREATE INDEX idx_op_request_id ON operation_logs (request_id);
CREATE INDEX idx_op_target ON operation_logs (target_type, target_id);
