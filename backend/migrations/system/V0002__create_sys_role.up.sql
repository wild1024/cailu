-- 系统角色表：RBAC 角色定义（如内置 SUPER_ADMIN）
CREATE TABLE sys_role (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    role_code VARCHAR(50) NOT NULL, -- 角色编码（全局唯一，如 SUPER_ADMIN）
    role_name VARCHAR(100) NOT NULL, -- 角色名称
    description VARCHAR(255), -- 角色描述
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    sort_order INT NOT NULL DEFAULT 0, -- 排序号（越小越靠前）
    remark TEXT, -- 备注
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_role IS '系统角色表：RBAC 角色定义（如内置 SUPER_ADMIN）';
COMMENT ON COLUMN sys_role.id IS '主键';
COMMENT ON COLUMN sys_role.role_code IS '角色编码（全局唯一，如 SUPER_ADMIN）';
COMMENT ON COLUMN sys_role.role_name IS '角色名称';
COMMENT ON COLUMN sys_role.description IS '角色描述';
COMMENT ON COLUMN sys_role.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_role.sort_order IS '排序号（越小越靠前）';
COMMENT ON COLUMN sys_role.remark IS '备注';
COMMENT ON COLUMN sys_role.created_at IS '创建时间';
COMMENT ON COLUMN sys_role.updated_at IS '更新时间';
COMMENT ON COLUMN sys_role.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_role.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_role.deleted_at IS '软删时间（非空即已删除）';

-- 角色编码唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_role_code ON sys_role (role_code) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_role_status ON sys_role (status);
