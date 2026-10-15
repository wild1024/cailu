-- 系统参数表：运行期可维护的键值配置
CREATE TABLE sys_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    config_key VARCHAR(100) NOT NULL, -- 参数键（全局唯一）
    config_value TEXT, -- 参数值
    config_type VARCHAR(20) NOT NULL DEFAULT 'STRING', -- 值类型：STRING / NUMBER / BOOLEAN / JSON
    description VARCHAR(255), -- 参数描述
    category VARCHAR(50), -- 参数分类
    sort_order INT NOT NULL DEFAULT 0, -- 排序号
    config_level SMALLINT NOT NULL DEFAULT 2, -- 层级：1 系统级 / 2 业务级
    is_modifiable BOOLEAN NOT NULL DEFAULT TRUE, -- 是否允许后台修改
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE, -- 是否加密存储
    is_restart BOOLEAN NOT NULL DEFAULT FALSE, -- 修改后是否需重启生效
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    ref_value VARCHAR(500), -- 推荐 / 参考值
    select_value VARCHAR(1000), -- 可选项列表（供前端下拉）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_config IS '系统参数表：运行期可维护的键值配置';
COMMENT ON COLUMN sys_config.id IS '主键';
COMMENT ON COLUMN sys_config.config_key IS '参数键（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_config.config_value IS '参数值（按 config_type 解释）';
COMMENT ON COLUMN sys_config.config_type IS '值类型：STRING / NUMBER / BOOLEAN / JSON';
COMMENT ON COLUMN sys_config.description IS '参数描述';
COMMENT ON COLUMN sys_config.category IS '参数分类';
COMMENT ON COLUMN sys_config.sort_order IS '排序号';
COMMENT ON COLUMN sys_config.config_level IS '层级：1 系统级 / 2 业务级';
COMMENT ON COLUMN sys_config.is_modifiable IS '是否允许后台修改';
COMMENT ON COLUMN sys_config.is_encrypted IS '是否加密存储';
COMMENT ON COLUMN sys_config.is_restart IS '修改后是否需重启生效';
COMMENT ON COLUMN sys_config.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_config.ref_value IS '推荐 / 参考值';
COMMENT ON COLUMN sys_config.select_value IS '可选项列表（供前端下拉）';
COMMENT ON COLUMN sys_config.created_at IS '创建时间';
COMMENT ON COLUMN sys_config.updated_at IS '更新时间';
COMMENT ON COLUMN sys_config.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_config.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_config.deleted_at IS '软删时间（非空即已删除）';

-- 参数键唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_config_key ON sys_config (config_key) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_config_category ON sys_config (category);
