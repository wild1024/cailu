-- ======================================================
-- 内置角色（种子迁移：指纹变化即重跑，故走 ON CONFLICT 幂等写入）
--
-- 仅内置一个不可删除的超级管理员角色；其权限由 role_code 短路判定，**不依赖**
-- sys_role_menu 授权表（见 src/system/auth/model.rs 的 is_super_admin）。
-- 文件末尾仍写入内置菜单的授权行，仅用于权限树 / 角色授权页的完整展示。
--
-- 管理员账号 admin 因初始密码需在代码中哈希（SQL 写不出），仍由
-- src/bootstrap.rs 生成，并在此角色就绪后完成账号与角色的绑定。
-- ======================================================

INSERT INTO sys_role (id, role_code, role_name, description, status, sort_order,
                      created_at, updated_at)
VALUES (gen_random_uuid(), 'SUPER_ADMIN', '超级管理员',
        '内置超级管理员角色，拥有全部权限且禁止删除', 1, 0,
        CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (role_code) WHERE deleted_at IS NULL
DO UPDATE SET role_name = EXCLUDED.role_name,
              description = EXCLUDED.description,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

-- ---------- 内置授权：超级管理员 ↔ 内置菜单 ----------
--
-- 超管的权限判定由 role_code 短路（见 src/system/auth/model.rs 的 is_super_admin），
-- 本段授权**并非鉴权所必需**；写入是为了让权限树 / 角色授权页能完整展示内置菜单。
--
-- 位置约束：sys_role_menu 对 sys_role(id) 与 sys_menu(id) 均有外键，且 seed 按名称升序
-- 执行（menu_data → role_data），因此必须写在**本文件**（角色行之后）、而菜单行由
-- S__menu_data.sql 先行写入；写进 S__menu_data.sql 会因角色尚未创建而违反外键。
INSERT INTO sys_role_menu (role_id, menu_id, created_at)
SELECT r.id, m.id, CURRENT_TIMESTAMP
FROM sys_role r
CROSS JOIN (VALUES ('10000000-0000-0000-0000-000000000050'::uuid),
                   ('10000000-0000-0000-0000-000000000051'::uuid)) AS m(id)
WHERE r.role_code = 'SUPER_ADMIN'
ON CONFLICT DO NOTHING;
