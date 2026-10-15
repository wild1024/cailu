/**
 * 接口模块统一出口。
 *
 * 各域接口按业务拆分在独立文件中，此处仅做汇聚，便于按域内聚、按需引用。
 */

export * from '@/api/system/auth-api';
export * from '@/api/system/user-api';
export * from '@/api/system/role-api';
export * from '@/api/system/menu-api';
export * from '@/api/system/dept-api';
export * from '@/api/support/dict-api';
export * from '@/api/support/config-api';
export * from '@/api/support/file-api';
export * from '@/api/support/log-api';
export * from '@/api/support/job-api';
export * from '@/api/support/online-api';
export * from '@/api/health';
