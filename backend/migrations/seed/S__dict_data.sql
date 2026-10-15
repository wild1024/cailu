-- ======================================================
-- 内置字典数据（种子迁移：指纹变化即重跑，故全部走 ON CONFLICT 幂等写入）
-- 字典编码与 src/shared/constants.rs 的 DICT_CODE_* 一一对应。
-- ======================================================

-- ---------- 字典类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'user_status', '用户状态', '后台账号的启用 / 停用状态', 1, 10)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'menu_type', '菜单类型', '菜单节点类型：目录 / 菜单 / 按钮', 1, 20)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'config_type', '参数类型', '系统参数的值类型', 1, 30)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'job_status', '任务状态', '定时任务的启用 / 停用状态', 1, 40)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：用户状态 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'user_status' AND deleted_at IS NULL),
        '1', '启用', 10, TRUE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'user_status' AND deleted_at IS NULL),
        '0', '停用', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：菜单类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'menu_type' AND deleted_at IS NULL),
        '1', '目录', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'menu_type' AND deleted_at IS NULL),
        '2', '菜单', 20, TRUE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'menu_type' AND deleted_at IS NULL),
        '3', '按钮', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：参数类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_type' AND deleted_at IS NULL),
        'STRING', '字符串', 10, TRUE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_type' AND deleted_at IS NULL),
        'NUMBER', '数字', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_type' AND deleted_at IS NULL),
        'BOOLEAN', '布尔', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_type' AND deleted_at IS NULL),
        'JSON', 'JSON', 40, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：任务状态 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'job_status' AND deleted_at IS NULL),
        '1', '启用', 10, TRUE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'job_status' AND deleted_at IS NULL),
        '0', '停用', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ======================================================
-- 以下为「页面展示中文对照」字典：
--   1) 配置管理页 tab 分类标签（sys_config.category -> 中文）
--   2) 日志页面编码字段的中文映射（认证类型 / 操作类型 / 目标类型 /
--      安全事件类型 / 密码变更类型）
-- 仍全部走 ON CONFLICT 幂等写入，增删条目只需修改本文件。
-- ======================================================

-- ---------- 字典类型：系统参数分类 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'config_category', '参数分类', '系统参数所属业务分类（配置管理页签）', 1, 50)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典类型：日志认证类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'log_auth_type', '认证类型', '认证日志的认证类型（登录 / 登出 / 刷新令牌）', 1, 60)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典类型：日志操作类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'log_operation', '日志操作类型', '审计 / 操作日志的操作类型编码对照', 1, 70)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典类型：日志目标类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'log_target_type', '日志目标类型', '审计 / 操作日志的操作对象类型对照', 1, 80)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典类型：安全事件类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'log_security_event', '安全事件类型', '安全日志的事件类型编码对照', 1, 90)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典类型：密码变更类型 ----------
INSERT INTO sys_dict_type (id, dict_code, dict_name, description, status, sort_order)
VALUES (gen_random_uuid(), 'log_password_change_type', '密码变更类型', '密码变更日志的变更类型编码对照', 1, 100)
ON CONFLICT (dict_code) WHERE deleted_at IS NULL
DO UPDATE SET dict_name = EXCLUDED.dict_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：系统参数分类 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_category' AND deleted_at IS NULL),
        'auth', '认证配置', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'config_category' AND deleted_at IS NULL),
        'log', '日志配置', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：日志认证类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_auth_type' AND deleted_at IS NULL),
        'LOGIN', '登录', 10, TRUE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_auth_type' AND deleted_at IS NULL),
        'LOGOUT', '登出', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_auth_type' AND deleted_at IS NULL),
        'REFRESH_TOKEN', '刷新令牌', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：日志操作类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'LOGIN_SUCCESS', '登录成功', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'PROFILE_UPDATE', '修改个人资料', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_CREATE', '新增用户', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_UPDATE', '修改用户', 40, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_DELETE', '删除用户', 50, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_STATUS', '修改用户状态', 60, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_PASSWORD_RESET', '重置用户密码', 70, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_ASSIGN_ROLE', '分配用户角色', 80, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'USER_BATCH_DEPT', '批量调整用户部门', 90, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'ROLE_CREATE', '新增角色', 100, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'ROLE_UPDATE', '修改角色', 110, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'ROLE_DELETE', '删除角色', 120, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'ROLE_ASSIGN_MENU', '分配角色菜单', 130, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'MENU_CREATE', '新增菜单', 140, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'MENU_UPDATE', '修改菜单', 150, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'MENU_DELETE', '删除菜单', 160, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DEPT_CREATE', '新增部门', 170, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DEPT_UPDATE', '修改部门', 180, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DEPT_DELETE', '删除部门', 190, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DEPT_MIGRATE_DELETE', '迁移并删除部门', 200, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'CONFIG_CREATE', '新增系统参数', 210, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'CONFIG_UPDATE', '修改系统参数', 220, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'CONFIG_DELETE', '删除系统参数', 230, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_TYPE_CREATE', '新增字典类型', 240, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_TYPE_UPDATE', '修改字典类型', 250, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_TYPE_DELETE', '删除字典类型', 260, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_ITEM_CREATE', '新增字典项', 270, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_ITEM_UPDATE', '修改字典项', 280, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'DICT_ITEM_DELETE', '删除字典项', 290, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'JOB_CREATE', '新增定时任务', 300, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'JOB_UPDATE', '修改定时任务', 310, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'JOB_DELETE', '删除定时任务', 320, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'JOB_STATUS', '修改定时任务状态', 330, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'JOB_TRIGGER', '立即执行定时任务', 340, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'FILE_UPLOAD', '上传文件', 350, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'FILE_UPLOAD_BATCH', '批量上传文件', 360, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_operation' AND deleted_at IS NULL),
        'FILE_DELETE', '删除文件', 370, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：日志目标类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'User', '用户', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Role', '角色', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Menu', '菜单', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Dept', '部门', 40, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Config', '系统参数', 50, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'DictType', '字典类型', 60, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'DictItem', '字典项', 70, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Job', '定时任务', 80, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'File', '文件', 90, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Auth', '认证', 100, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Online', '在线会话', 110, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_target_type' AND deleted_at IS NULL),
        'Log', '日志', 120, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：安全事件类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_security_event' AND deleted_at IS NULL),
        'LOGIN_FAIL', '登录失败', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_security_event' AND deleted_at IS NULL),
        'PERMISSION_DENY', '权限拒绝', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_security_event' AND deleted_at IS NULL),
        'SQL_INJECT_ATTEMPT', 'SQL 注入尝试', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 字典项：密码变更类型 ----------
INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_password_change_type' AND deleted_at IS NULL),
        'RESET', '重置密码', 10, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_password_change_type' AND deleted_at IS NULL),
        'UPDATE', '修改密码', 20, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_dict_item (id, dict_type_id, item_value, item_label, sort_order, is_default, status)
VALUES (gen_random_uuid(),
        (SELECT id FROM sys_dict_type WHERE dict_code = 'log_password_change_type' AND deleted_at IS NULL),
        'FORCED', '强制改密', 30, FALSE, 1)
ON CONFLICT (dict_type_id, item_value) WHERE deleted_at IS NULL
DO UPDATE SET item_label = EXCLUDED.item_label,
              sort_order = EXCLUDED.sort_order,
              is_default = EXCLUDED.is_default,
              status = EXCLUDED.status,
              updated_at = CURRENT_TIMESTAMP;
