-- 认证日志表：登录 / 登出 / 刷新令牌的认证结果（只写不改删）
CREATE TABLE auth_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    request_id VARCHAR(36) NOT NULL, -- 请求追踪 ID
    user_id UUID, -- 登录用户 ID（用户不存在时为空）
    username VARCHAR(50), -- 登录用户名
    auth_type VARCHAR(20) NOT NULL, -- 认证类型：LOGIN / LOGOUT / REFRESH_TOKEN
    auth_status VARCHAR(20) NOT NULL, -- 认证状态：SUCCESS / FAIL
    ip INET, -- 客户端 IP
    location VARCHAR(255), -- 登录地点（由 IP 解析，未启用解析时为空）
    device_type VARCHAR(50), -- 设备类型：PC / MOBILE / TABLET
    browser_type VARCHAR(100), -- 浏览器类型
    os_type VARCHAR(50), -- 操作系统类型
    user_agent TEXT, -- 完整 User-Agent
    error_message VARCHAR(255), -- 失败原因（成功时为空）
    login_duration INT, -- 登录耗时（毫秒）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

COMMENT ON TABLE auth_logs IS '认证日志表：登录 / 登出 / 刷新令牌的认证结果（只写不改删）';
COMMENT ON COLUMN auth_logs.id IS '主键';
COMMENT ON COLUMN auth_logs.request_id IS '请求追踪 ID';
COMMENT ON COLUMN auth_logs.user_id IS '登录用户 ID（用户不存在时为空）';
COMMENT ON COLUMN auth_logs.username IS '登录用户名';
COMMENT ON COLUMN auth_logs.auth_type IS '认证类型：LOGIN / LOGOUT / REFRESH_TOKEN';
COMMENT ON COLUMN auth_logs.auth_status IS '认证状态：SUCCESS / FAIL';
COMMENT ON COLUMN auth_logs.ip IS '客户端 IP';
COMMENT ON COLUMN auth_logs.location IS '登录地点（由 IP 解析，未启用解析时为空）';
COMMENT ON COLUMN auth_logs.device_type IS '设备类型：PC / MOBILE / TABLET';
COMMENT ON COLUMN auth_logs.browser_type IS '浏览器类型';
COMMENT ON COLUMN auth_logs.os_type IS '操作系统类型';
COMMENT ON COLUMN auth_logs.user_agent IS '完整 User-Agent';
COMMENT ON COLUMN auth_logs.error_message IS '失败原因（成功时为空）';
COMMENT ON COLUMN auth_logs.login_duration IS '登录耗时（毫秒）';
COMMENT ON COLUMN auth_logs.created_at IS '创建时间（认证发生时刻）';

CREATE INDEX idx_auth_user_id ON auth_logs (user_id);
CREATE INDEX idx_auth_username ON auth_logs (username);
CREATE INDEX idx_auth_created_at ON auth_logs (created_at DESC);
