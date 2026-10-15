-- 字典项表：归属某个字典类型的具体可选项
CREATE TABLE sys_dict_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    dict_type_id UUID NOT NULL REFERENCES sys_dict_type(id), -- 所属字典类型 ID
    item_value VARCHAR(100) NOT NULL, -- 字典项取值
    item_label VARCHAR(200) NOT NULL, -- 字典项显示标签
    description VARCHAR(255), -- 字典项描述
    sort_order INT NOT NULL DEFAULT 0, -- 排序号
    is_default BOOLEAN NOT NULL DEFAULT FALSE, -- 是否为默认项
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_dict_item IS '字典项表：归属某个字典类型的具体可选项';
COMMENT ON COLUMN sys_dict_item.id IS '主键';
COMMENT ON COLUMN sys_dict_item.dict_type_id IS '所属字典类型 ID（sys_dict_type.id）';
COMMENT ON COLUMN sys_dict_item.item_value IS '字典项取值（同一字典类型内唯一）';
COMMENT ON COLUMN sys_dict_item.item_label IS '字典项显示标签';
COMMENT ON COLUMN sys_dict_item.description IS '字典项描述';
COMMENT ON COLUMN sys_dict_item.sort_order IS '排序号';
COMMENT ON COLUMN sys_dict_item.is_default IS '是否为默认项';
COMMENT ON COLUMN sys_dict_item.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_dict_item.created_at IS '创建时间';
COMMENT ON COLUMN sys_dict_item.updated_at IS '更新时间';
COMMENT ON COLUMN sys_dict_item.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_dict_item.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_dict_item.deleted_at IS '软删时间（非空即已删除）';

CREATE INDEX idx_sys_dict_item_type_id ON sys_dict_item (dict_type_id);
-- 同一字典类型内取值唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_dict_item_value ON sys_dict_item (dict_type_id, item_value) WHERE deleted_at IS NULL;
