-- ======================================================
-- 内置系统参数（种子迁移：指纹变化即重跑，故走 ON CONFLICT 幂等写入）
-- 登录安全项与 src/shared/constants.rs 的默认常量对应，登录流程优先读库、
-- 读不到时回退常量，因此这两行是「可调默认值」而非唯一来源。
-- ======================================================

INSERT INTO sys_config (id, config_key, config_value, config_type, description,
                        category, config_level, is_modifiable, is_encrypted, is_restart, status, sort_order)
VALUES (gen_random_uuid(), 'auth.max_login_fail_count', '5', 'NUMBER',
        '连续登录失败达到该次数即锁定账号', 'auth', 1, TRUE, FALSE, FALSE, 1, 10)
ON CONFLICT (config_key) WHERE deleted_at IS NULL
DO UPDATE SET config_value = EXCLUDED.config_value,
              config_type = EXCLUDED.config_type,
              description = EXCLUDED.description,
              category = EXCLUDED.category,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_config (id, config_key, config_value, config_type, description,
                        category, config_level, is_modifiable, is_encrypted, is_restart, status, sort_order)
VALUES (gen_random_uuid(), 'auth.lock_minutes', '30', 'NUMBER',
        '账号锁定后自动解锁的等待时长（分钟）', 'auth', 1, TRUE, FALSE, FALSE, 1, 20)
ON CONFLICT (config_key) WHERE deleted_at IS NULL
DO UPDATE SET config_value = EXCLUDED.config_value,
              config_type = EXCLUDED.config_type,
              description = EXCLUDED.description,
              category = EXCLUDED.category,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;

INSERT INTO sys_config (id, config_key, config_value, config_type, description,
                        category, config_level, is_modifiable, is_encrypted, is_restart, status, sort_order)
VALUES (gen_random_uuid(), 'log.ip_location_enabled', 'true', 'BOOL',
        '是否解析登录日志的 IP 归属地（关闭后不再外呼，仅保留本地网络标识）', 'log', 1, TRUE, FALSE, FALSE, 1, 30)
ON CONFLICT (config_key) WHERE deleted_at IS NULL
DO UPDATE SET config_value = EXCLUDED.config_value,
              config_type = EXCLUDED.config_type,
              description = EXCLUDED.description,
              category = EXCLUDED.category,
              status = EXCLUDED.status,
              sort_order = EXCLUDED.sort_order,
              updated_at = CURRENT_TIMESTAMP;
