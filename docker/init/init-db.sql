-- 开发依赖初始化脚本：创建测试库（由 postgres 镜像在空数据卷首次启动时执行）
-- 开发库 cailu 由 POSTGRES_DB 环境变量自动创建，这里仅补充测试库 cailu_test。
CREATE DATABASE cailu_test OWNER alun;
