-- 系统菜单表：目录 / 菜单 / 按钮三级结构，按钮即权限点
CREATE TABLE sys_menu (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    name VARCHAR(100) NOT NULL, -- 菜单名称，同时作为种子幂等的唯一键
    menu_type SMALLINT NOT NULL DEFAULT 2, -- 类型：1 目录 / 2 菜单 / 3 按钮
    auth_code VARCHAR(100) NOT NULL, -- 权限编码，格式 {模块}:{资源}:{动作}
    path VARCHAR(255), -- 前端路由地址（按钮为空）
    parent_id UUID, -- 父菜单 ID（根节点为空）
    component VARCHAR(255), -- 前端组件路径（目录固定为 Layout，按钮为空）
    sort_order INT NOT NULL DEFAULT 0, -- 同级排序号
    status SMALLINT NOT NULL DEFAULT 1, -- 状态：1 启用 / 0 停用
    icon VARCHAR(100), -- 图标名（TDesign 图标）
    title VARCHAR(100), -- 界面显示标题
    is_frame BOOLEAN NOT NULL DEFAULT FALSE, -- 是否外链
    is_cache BOOLEAN NOT NULL DEFAULT FALSE, -- 是否页面缓存
    is_visible BOOLEAN NOT NULL DEFAULT TRUE, -- 是否在导航中可见（按钮不可见）
    hidden_expression VARCHAR(255), -- 隐藏表达式：条件成立时前端隐藏该项（NULL 表示不隐藏）
    module_code VARCHAR(20), -- 所属模块编码（system / support）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 更新时间
    create_by UUID, -- 创建人 ID
    update_by UUID, -- 更新人 ID
    deleted_at TIMESTAMPTZ -- 软删时间（非空即已删除）
);

COMMENT ON TABLE sys_menu IS '系统菜单表：目录 / 菜单 / 按钮三级结构，按钮即权限点';
COMMENT ON COLUMN sys_menu.id IS '主键';
COMMENT ON COLUMN sys_menu.name IS '菜单名称，同时作为种子幂等的唯一键';
COMMENT ON COLUMN sys_menu.menu_type IS '类型：1 目录 / 2 菜单 / 3 按钮';
COMMENT ON COLUMN sys_menu.auth_code IS '权限编码，格式 {模块}:{资源}:{动作}';
COMMENT ON COLUMN sys_menu.path IS '前端路由地址（按钮为空）';
COMMENT ON COLUMN sys_menu.parent_id IS '父菜单 ID（根节点为空）';
COMMENT ON COLUMN sys_menu.component IS '前端组件路径（目录固定为 Layout，按钮为空）';
COMMENT ON COLUMN sys_menu.sort_order IS '同级排序号';
COMMENT ON COLUMN sys_menu.status IS '状态：1 启用 / 0 停用';
COMMENT ON COLUMN sys_menu.icon IS '图标名（TDesign 图标）';
COMMENT ON COLUMN sys_menu.title IS '界面显示标题';
COMMENT ON COLUMN sys_menu.is_frame IS '是否外链';
COMMENT ON COLUMN sys_menu.is_cache IS '是否页面缓存';
COMMENT ON COLUMN sys_menu.is_visible IS '是否在导航中可见（按钮不可见）';
COMMENT ON COLUMN sys_menu.hidden_expression IS '隐藏表达式：条件成立时前端隐藏该项（NULL 表示不隐藏）';
COMMENT ON COLUMN sys_menu.module_code IS '所属模块编码（system / support）';
COMMENT ON COLUMN sys_menu.created_at IS '创建时间';
COMMENT ON COLUMN sys_menu.updated_at IS '更新时间';
COMMENT ON COLUMN sys_menu.create_by IS '创建人 ID';
COMMENT ON COLUMN sys_menu.update_by IS '更新人 ID';
COMMENT ON COLUMN sys_menu.deleted_at IS '软删时间（非空即已删除）';

CREATE INDEX idx_sys_menu_parent_id ON sys_menu (parent_id);
CREATE INDEX idx_sys_menu_status ON sys_menu (status);
CREATE INDEX idx_sys_menu_auth_code ON sys_menu (auth_code);
CREATE INDEX idx_sys_menu_module_code ON sys_menu (module_code);
