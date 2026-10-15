-- 字典类型表：一组字典项的归类（如「用户状态」）
CREATE TABLE sys_dict_type (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    dict_code VARCHAR(50) NOT NULL, -- 字典编码（全局唯一）
    dict_name VARCHAR(100) NOT NULL, -- 字典名称
    description VARCHAR(255), -- 字典描述
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    sort_order INT NOT NULL DEFAULT 0, -- 排序号
    remark TEXT, -- 备注
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_dict_type IS '字典类型表：一组字典项的归类（如「用户状态」）';
COMMENT ON COLUMN sys_dict_type.id IS '主键';
COMMENT ON COLUMN sys_dict_type.dict_code IS '字典编码（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_dict_type.dict_name IS '字典名称';
COMMENT ON COLUMN sys_dict_type.description IS '字典描述';
COMMENT ON COLUMN sys_dict_type.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_dict_type.sort_order IS '排序号';
COMMENT ON COLUMN sys_dict_type.remark IS '备注';
COMMENT ON COLUMN sys_dict_type.created_at IS '创建时间';
COMMENT ON COLUMN sys_dict_type.updated_at IS '更新时间';
COMMENT ON COLUMN sys_dict_type.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_dict_type.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_dict_type.deleted_at IS '软删时间（非空即已删除）';

-- 字典编码唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_dict_type_code ON sys_dict_type (dict_code) WHERE deleted_at IS NULL;
