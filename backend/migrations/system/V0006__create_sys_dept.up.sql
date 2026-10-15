-- 系统部门表：组织架构树，dept_path 缓存层级路径便于子树查询
CREATE TABLE sys_dept (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    dept_code VARCHAR(50) NOT NULL, -- 部门编码（全局唯一）
    dept_name VARCHAR(100) NOT NULL, -- 部门名称
    parent_id UUID, -- 上级部门 ID（根节点为空）
    dept_path VARCHAR(500), -- 层级路径（如 /1/2/，加速子树查询）
    leader_id UUID, -- 负责人用户 ID（sys_user.id）
    phone VARCHAR(20), -- 联系电话
    email VARCHAR(100), -- 联系邮箱
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    sort_order INT NOT NULL DEFAULT 0, -- 同级排序号
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_dept IS '系统部门表：组织架构树，dept_path 缓存层级路径便于子树查询';
COMMENT ON COLUMN sys_dept.id IS '主键';
COMMENT ON COLUMN sys_dept.dept_code IS '部门编码（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_dept.dept_name IS '部门名称';
COMMENT ON COLUMN sys_dept.parent_id IS '上级部门 ID（根节点为空）';
COMMENT ON COLUMN sys_dept.dept_path IS '层级路径（如 /1/2/，加速子树查询）';
COMMENT ON COLUMN sys_dept.leader_id IS '负责人用户 ID（sys_user.id）';
COMMENT ON COLUMN sys_dept.phone IS '联系电话';
COMMENT ON COLUMN sys_dept.email IS '联系邮箱';
COMMENT ON COLUMN sys_dept.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_dept.sort_order IS '同级排序号';
COMMENT ON COLUMN sys_dept.created_at IS '创建时间';
COMMENT ON COLUMN sys_dept.updated_at IS '更新时间';
COMMENT ON COLUMN sys_dept.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_dept.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_dept.deleted_at IS '软删时间（非空即已删除）';

-- 部门编码唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_dept_code ON sys_dept (dept_code) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_dept_parent_id ON sys_dept (parent_id);
CREATE INDEX idx_sys_dept_path ON sys_dept (dept_path);
