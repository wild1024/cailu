-- 文件记录表：上传文件的元数据（file_hash 支持秒传判定）
CREATE TABLE sys_file (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    file_name VARCHAR(255) NOT NULL, -- 存储文件名
    original_name VARCHAR(255), -- 上传时的原始文件名
    file_key VARCHAR(500) NOT NULL, -- 对象存储键 / 业务唯一标识（全局唯一）
    file_size BIGINT NOT NULL, -- 文件大小（字节）
    file_hash VARCHAR(64), -- 内容 SHA-256（用于秒传与完整性校验）
    mime_type VARCHAR(100), -- MIME 类型
    extension VARCHAR(50), -- 文件扩展名
    storage_path VARCHAR(500), -- 存储路径
    category VARCHAR(50), -- 业务分类
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 正常 / 0 停用
    uploaded_by UUID, -- 上传人 ID（sys_user.id）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_file IS '文件记录表：上传文件的元数据（file_hash 支持秒传判定）';
COMMENT ON COLUMN sys_file.id IS '主键';
COMMENT ON COLUMN sys_file.file_name IS '存储文件名';
COMMENT ON COLUMN sys_file.original_name IS '上传时的原始文件名';
COMMENT ON COLUMN sys_file.file_key IS '对象存储键 / 业务唯一标识（全局唯一，软删后可复用）';
COMMENT ON COLUMN sys_file.file_size IS '文件大小（字节）';
COMMENT ON COLUMN sys_file.file_hash IS '内容 SHA-256（用于秒传与完整性校验）';
COMMENT ON COLUMN sys_file.mime_type IS 'MIME 类型';
COMMENT ON COLUMN sys_file.extension IS '文件扩展名';
COMMENT ON COLUMN sys_file.storage_path IS '存储路径';
COMMENT ON COLUMN sys_file.category IS '业务分类';
COMMENT ON COLUMN sys_file.status IS '状态：1 正常 / 0 停用';
COMMENT ON COLUMN sys_file.uploaded_by IS '上传人 ID（sys_user.id）';
COMMENT ON COLUMN sys_file.created_at IS '创建时间';
COMMENT ON COLUMN sys_file.updated_at IS '更新时间';
COMMENT ON COLUMN sys_file.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_file.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_file.deleted_at IS '软删时间（非空即已删除）';

-- 文件标识唯一：仅约束未软删行
CREATE UNIQUE INDEX uk_sys_file_key ON sys_file (file_key) WHERE deleted_at IS NULL;
CREATE INDEX idx_sys_file_category ON sys_file (category);
CREATE INDEX idx_sys_file_uploaded_by ON sys_file (uploaded_by);
