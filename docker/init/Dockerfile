# 菜芦开发依赖 —— PostgreSQL 镜像（内置建库初始化脚本）
#
# 背景：Docker Desktop 对部分外部卷路径（如 /Volumes/*）的单文件 bind 挂载存在
# 「文件被映射为目录」的兼容性问题，导致 docker-entrypoint-initdb.d 初始化脚本
# 无法执行。此处改为把脚本构建进镜像，绕开 bind 挂载，任何宿主机路径均可稳定工作。
FROM postgres:16-alpine

# 初始化脚本：仅创建测试库 cailu_test（开发库 cailu 由 POSTGRES_DB 环境变量创建）
COPY init-db.sql /docker-entrypoint-initdb.d/10-init-db.sql
