-- ======================================================
-- 内置菜单与按钮权限点（种子迁移：指纹变化即重跑，故走 ON CONFLICT 幂等写入）
--
-- 与前端静态路由表、后端 #[alun::permission] 声明的权限码一一对应；权限码格式
-- 为 {模块}:{资源}:{动作}，改动此处须同步处理器上的声明值，否则按钮级鉴权失效。
--
-- 幂等说明：
--   * id 使用固定 UUID 字面量，故 ON CONFLICT (id) 可在重跑时自愈（改名 / 调序 /
--     改标题均会被覆盖回种子值）；
--   * parent_id 直接引用父节点的固定 UUID，无需依赖子查询或运行时分配主键；
--   * 新增功能只需在本文件追加一组 VALUES 行（含其父节点 id），不再改 Rust 代码。
--
-- 列顺序：id, name, menu_type, auth_code, path, parent_id, component,
--         sort_order, status, icon, title, is_visible, module_code
-- menu_type：1 目录 / 2 菜单 / 3 按钮；按钮不参与导航渲染故 is_visible = FALSE。
-- ======================================================

INSERT INTO sys_menu (id, name, menu_type, auth_code, path, parent_id, component,
                      sort_order, status, icon, title, is_visible, module_code)
SELECT v.id, v.name, v.menu_type, v.auth_code, v.path, v.parent_id, v.component,
       v.sort_order, v.status, v.icon, v.title, v.is_visible, v.module_code
FROM (VALUES
    -- ---------- 系统管理目录 ----------
    ('10000000-0000-0000-0000-000000000001'::uuid, 'system', 1, '', '/system', NULL::uuid,
     'Layout', 10, 1, 'setting', '系统管理', TRUE, 'system'),

    -- ---------- 系统管理 / 用户管理 ----------
    ('10000000-0000-0000-0000-000000000002'::uuid, 'system:user', 2, 'system:user:list',
     '/system/user', '10000000-0000-0000-0000-000000000001'::uuid, 'system/user/index',
     10, 1, 'user', '用户管理', TRUE, 'system'),
    ('10000000-0000-0000-0000-000000000003'::uuid, 'system:user:create', 3, 'system:user:create',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 10, 1, NULL, '新增用户', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000004'::uuid, 'system:user:update', 3, 'system:user:update',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 20, 1, NULL, '编辑用户', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000005'::uuid, 'system:user:delete', 3, 'system:user:delete',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 30, 1, NULL, '删除用户', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000006'::uuid, 'system:user:status', 3, 'system:user:status',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 40, 1, NULL, '启用 / 停用用户', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000007'::uuid, 'system:user:password', 3, 'system:user:password',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 50, 1, NULL, '重置密码', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000008'::uuid, 'system:user:role', 3, 'system:user:role',
     NULL, '10000000-0000-0000-0000-000000000002'::uuid, NULL, 60, 1, NULL, '分配角色', FALSE, 'system'),

    -- ---------- 系统管理 / 角色管理 ----------
    ('10000000-0000-0000-0000-000000000009'::uuid, 'system:role', 2, 'system:role:list',
     '/system/role', '10000000-0000-0000-0000-000000000001'::uuid, 'system/role/index',
     20, 1, 'usergroup', '角色管理', TRUE, 'system'),
    ('10000000-0000-0000-0000-000000000010'::uuid, 'system:role:create', 3, 'system:role:create',
     NULL, '10000000-0000-0000-0000-000000000009'::uuid, NULL, 10, 1, NULL, '新增角色', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000011'::uuid, 'system:role:update', 3, 'system:role:update',
     NULL, '10000000-0000-0000-0000-000000000009'::uuid, NULL, 20, 1, NULL, '编辑角色', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000012'::uuid, 'system:role:delete', 3, 'system:role:delete',
     NULL, '10000000-0000-0000-0000-000000000009'::uuid, NULL, 30, 1, NULL, '删除角色', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000013'::uuid, 'system:role:menu', 3, 'system:role:menu',
     NULL, '10000000-0000-0000-0000-000000000009'::uuid, NULL, 40, 1, NULL, '分配菜单权限', FALSE, 'system'),

    -- ---------- 系统管理 / 菜单管理 ----------
    ('10000000-0000-0000-0000-000000000014'::uuid, 'system:menu', 2, 'system:menu:list',
     '/system/menu', '10000000-0000-0000-0000-000000000001'::uuid, 'system/menu/index',
     30, 1, 'menu-application', '菜单管理', TRUE, 'system'),
    ('10000000-0000-0000-0000-000000000015'::uuid, 'system:menu:create', 3, 'system:menu:create',
     NULL, '10000000-0000-0000-0000-000000000014'::uuid, NULL, 10, 1, NULL, '新增菜单', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000016'::uuid, 'system:menu:update', 3, 'system:menu:update',
     NULL, '10000000-0000-0000-0000-000000000014'::uuid, NULL, 20, 1, NULL, '编辑菜单', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000017'::uuid, 'system:menu:delete', 3, 'system:menu:delete',
     NULL, '10000000-0000-0000-0000-000000000014'::uuid, NULL, 30, 1, NULL, '删除菜单', FALSE, 'system'),

    -- ---------- 系统管理 / 部门管理 ----------
    ('10000000-0000-0000-0000-000000000018'::uuid, 'system:dept', 2, 'system:dept:list',
     '/system/dept', '10000000-0000-0000-0000-000000000001'::uuid, 'system/dept/index',
     40, 1, 'tree-round-dot', '部门管理', TRUE, 'system'),
    ('10000000-0000-0000-0000-000000000019'::uuid, 'system:dept:create', 3, 'system:dept:create',
     NULL, '10000000-0000-0000-0000-000000000018'::uuid, NULL, 10, 1, NULL, '新增部门', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000020'::uuid, 'system:dept:update', 3, 'system:dept:update',
     NULL, '10000000-0000-0000-0000-000000000018'::uuid, NULL, 20, 1, NULL, '编辑部门', FALSE, 'system'),
    ('10000000-0000-0000-0000-000000000021'::uuid, 'system:dept:delete', 3, 'system:dept:delete',
     NULL, '10000000-0000-0000-0000-000000000018'::uuid, NULL, 30, 1, NULL, '删除部门', FALSE, 'system'),

    -- ---------- 支撑功能目录 ----------
    ('10000000-0000-0000-0000-000000000022'::uuid, 'support', 1, '', '/support', NULL::uuid,
     'Layout', 20, 1, 'app', '支撑功能', TRUE, 'support'),

    -- ---------- 支撑功能 / 字典管理 ----------
    ('10000000-0000-0000-0000-000000000023'::uuid, 'support:dict', 2, 'support:dict:list',
     '/support/dict', '10000000-0000-0000-0000-000000000022'::uuid, 'support/dict/index',
     10, 1, 'book', '字典管理', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000024'::uuid, 'support:dict:create', 3, 'support:dict:create',
     NULL, '10000000-0000-0000-0000-000000000023'::uuid, NULL, 10, 1, NULL, '新增字典', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000025'::uuid, 'support:dict:update', 3, 'support:dict:update',
     NULL, '10000000-0000-0000-0000-000000000023'::uuid, NULL, 20, 1, NULL, '编辑字典', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000026'::uuid, 'support:dict:delete', 3, 'support:dict:delete',
     NULL, '10000000-0000-0000-0000-000000000023'::uuid, NULL, 30, 1, NULL, '删除字典', FALSE, 'support'),

    -- ---------- 支撑功能 / 参数配置 ----------
    ('10000000-0000-0000-0000-000000000027'::uuid, 'support:config', 2, 'support:config:list',
     '/support/config', '10000000-0000-0000-0000-000000000022'::uuid, 'support/config/index',
     20, 1, 'setting-1', '参数配置', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000028'::uuid, 'support:config:create', 3, 'support:config:create',
     NULL, '10000000-0000-0000-0000-000000000027'::uuid, NULL, 10, 1, NULL, '新增参数', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000029'::uuid, 'support:config:update', 3, 'support:config:update',
     NULL, '10000000-0000-0000-0000-000000000027'::uuid, NULL, 20, 1, NULL, '编辑参数', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000030'::uuid, 'support:config:delete', 3, 'support:config:delete',
     NULL, '10000000-0000-0000-0000-000000000027'::uuid, NULL, 30, 1, NULL, '删除参数', FALSE, 'support'),

    -- ---------- 支撑功能 / 文件管理 ----------
    ('10000000-0000-0000-0000-000000000031'::uuid, 'support:file', 2, 'support:file:list',
     '/support/file', '10000000-0000-0000-0000-000000000022'::uuid, 'support/file/index',
     30, 1, 'file', '文件管理', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000032'::uuid, 'support:file:upload', 3, 'support:file:upload',
     NULL, '10000000-0000-0000-0000-000000000031'::uuid, NULL, 10, 1, NULL, '上传文件', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000033'::uuid, 'support:file:delete', 3, 'support:file:delete',
     NULL, '10000000-0000-0000-0000-000000000031'::uuid, NULL, 20, 1, NULL, '删除文件', FALSE, 'support'),

    -- ---------- 支撑功能 / 操作日志 ----------
    ('10000000-0000-0000-0000-000000000034'::uuid, 'support:log', 2, 'support:log:list',
     '/support/log', '10000000-0000-0000-0000-000000000022'::uuid, 'support/log/index',
     40, 1, 'file-copy', '操作日志', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000035'::uuid, 'support:log:delete', 3, 'support:log:delete',
     NULL, '10000000-0000-0000-0000-000000000034'::uuid, NULL, 10, 1, NULL, '清理日志', FALSE, 'support'),

    -- ---------- 支撑功能 / 定时任务 ----------
    ('10000000-0000-0000-0000-000000000036'::uuid, 'support:job', 2, 'support:job:list',
     '/support/job', '10000000-0000-0000-0000-000000000022'::uuid, 'support/job/index',
     50, 1, 'time', '定时任务', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000037'::uuid, 'support:job:create', 3, 'support:job:create',
     NULL, '10000000-0000-0000-0000-000000000036'::uuid, NULL, 10, 1, NULL, '新增任务', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000038'::uuid, 'support:job:update', 3, 'support:job:update',
     NULL, '10000000-0000-0000-0000-000000000036'::uuid, NULL, 20, 1, NULL, '编辑任务', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000039'::uuid, 'support:job:delete', 3, 'support:job:delete',
     NULL, '10000000-0000-0000-0000-000000000036'::uuid, NULL, 30, 1, NULL, '删除任务', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000040'::uuid, 'support:job:trigger', 3, 'support:job:trigger',
     NULL, '10000000-0000-0000-0000-000000000036'::uuid, NULL, 40, 1, NULL, '手动执行任务', FALSE, 'support'),

    -- ---------- 支撑功能 / 在线用户 ----------
    ('10000000-0000-0000-0000-000000000041'::uuid, 'support:online', 2, 'support:online:list',
     '/support/online', '10000000-0000-0000-0000-000000000022'::uuid, 'support/online/index',
     60, 1, 'user', '在线用户', TRUE, 'support'),
    ('10000000-0000-0000-0000-000000000042'::uuid, 'support:online:force', 3, 'support:online:force',
     NULL, '10000000-0000-0000-0000-000000000041'::uuid, NULL, 10, 1, NULL, '强制下线', FALSE, 'support'),

    -- ---------- 工作台（登录后首页，隐藏项：作为落地页而非导航项） ----------
    -- 首页此前没有菜单记录，导致工作台聚合接口无权限点可挂，故补一条目录 + 一个按钮级权限点。
    ('10000000-0000-0000-0000-000000000050'::uuid, '工作台', 2, 'support:dashboard:list',
     '/dashboard', NULL::uuid, 'dashboard/index', 5, 1, 'dashboard', '工作台', FALSE, 'support'),
    ('10000000-0000-0000-0000-000000000051'::uuid, '工作台概览', 3, 'support:dashboard:list',
     NULL, '10000000-0000-0000-0000-000000000050'::uuid, NULL, 10, 1, NULL, NULL, TRUE, 'support')
) AS v(id, name, menu_type, auth_code, path, parent_id, component,
       sort_order, status, icon, title, is_visible, module_code)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    menu_type = EXCLUDED.menu_type,
    auth_code = EXCLUDED.auth_code,
    path = EXCLUDED.path,
    parent_id = EXCLUDED.parent_id,
    component = EXCLUDED.component,
    sort_order = EXCLUDED.sort_order,
    status = EXCLUDED.status,
    icon = EXCLUDED.icon,
    title = EXCLUDED.title,
    is_visible = EXCLUDED.is_visible,
    module_code = EXCLUDED.module_code,
    updated_at = CURRENT_TIMESTAMP;
