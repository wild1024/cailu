-- 角色-菜单关联表：角色被授予的菜单 / 权限点
CREATE TABLE sys_role_menu (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 主键
    role_id UUID NOT NULL REFERENCES sys_role(id), -- 角色 ID（sys_role.id）
    menu_id UUID NOT NULL REFERENCES sys_menu(id), -- 菜单 ID（sys_menu.id）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 创建时间
    UNIQUE (role_id, menu_id) -- 同一角色对同一菜单只允许一条授权
);

COMMENT ON TABLE sys_role_menu IS '角色-菜单关联表：角色被授予的菜单 / 权限点';
COMMENT ON COLUMN sys_role_menu.id IS '主键';
COMMENT ON COLUMN sys_role_menu.role_id IS '角色 ID（sys_role.id）';
COMMENT ON COLUMN sys_role_menu.menu_id IS '菜单 ID（sys_menu.id）';
COMMENT ON COLUMN sys_role_menu.created_at IS '创建时间';

CREATE INDEX idx_sys_role_menu_role_id ON sys_role_menu (role_id);
CREATE INDEX idx_sys_role_menu_menu_id ON sys_role_menu (menu_id);
