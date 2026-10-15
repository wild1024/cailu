-- 用户-角色关联表：一个用户可绑定多个角色
CREATE TABLE sys_user_role (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    user_id UUID NOT NULL REFERENCES sys_user(id), -- 用户 ID（sys_user.id）
    role_id UUID NOT NULL REFERENCES sys_role(id), -- 角色 ID（sys_role.id）
    expire_time TIMESTAMPTZ, -- 授权过期时间（NULL 表示永久有效）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    UNIQUE (user_id, role_id) -- 同一用户对同一角色只允许一条绑定
);

COMMENT ON TABLE sys_user_role IS '用户-角色关联表：一个用户可绑定多个角色';
COMMENT ON COLUMN sys_user_role.id IS '主键';
COMMENT ON COLUMN sys_user_role.user_id IS '用户 ID（sys_user.id）';
COMMENT ON COLUMN sys_user_role.role_id IS '角色 ID（sys_role.id）';
COMMENT ON COLUMN sys_user_role.expire_time IS '授权过期时间（NULL 表示永久有效）';
COMMENT ON COLUMN sys_user_role.created_at IS '创建时间';

CREATE INDEX idx_sys_user_role_user_id ON sys_user_role (user_id);
CREATE INDEX idx_sys_user_role_role_id ON sys_user_role (role_id);
