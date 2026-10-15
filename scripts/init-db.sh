#!/usr/bin/env bash
# 初始化 PostgreSQL 数据库：cailu（开发库）/ cailu_test（测试库）
#
# 用途：
#   * 不使用 docker、直连本机 PG 时，先执行本脚本建库；
#   * 使用 docker compose 依赖时无需执行（docker/docker-compose.yml 自动创建）；
#   * 本机无 psql 客户端时，自动回退用 docker compose 的 postgres 服务执行。
#
# 用法：
#   bash scripts/init-db.sh                          # 默认连 127.0.0.1:5432，账号 alun
#   PGPASSWORD=xxx bash scripts/init-db.sh           # 需要密码时注入
#   PGUSER=postgres bash scripts/init-db.sh           # 换连接账号
#   PGHOST=... PGPORT=... PGUSER=... bash scripts/init-db.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-alun}"
DB_DEV="${DB_DEV:-cailu}"
DB_TEST="${DB_TEST:-cailu_test}"

# 执行通道：优先本机 psql，缺失时回退 docker compose 的 postgres 服务
use_docker=false
if command -v psql >/dev/null 2>&1; then
    use_docker=false
elif command -v docker >/dev/null 2>&1 \
    && [ -n "$(docker compose -f "$ROOT/docker/docker-compose.yml" ps -q postgres 2>/dev/null)" ]; then
    echo "==> 本机无 psql，改用 docker compose 的 postgres 服务执行"
    use_docker=true
else
    echo "错误：找不到 psql 客户端，且 docker compose 的 postgres 服务未运行。" >&2
    echo "请安装 postgresql-client，或先执行 docker compose -f docker/docker-compose.yml up -d。" >&2
    exit 1
fi

# 执行一条 SQL（输出到 stdout），本地 psql 与 docker 通道共用
run_sql() {
    local sql="$1"
    if [ "$use_docker" = true ]; then
        docker compose -f "$ROOT/docker/docker-compose.yml" exec -T postgres \
            psql -U "$PGUSER" -d postgres -tA -v ON_ERROR_STOP=1 -c "$sql"
    else
        psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d postgres -tA -v ON_ERROR_STOP=1 -c "$sql"
    fi
}

# 库不存在时创建（CREATE DATABASE 无 IF NOT EXISTS，需先探测）
create_db_if_missing() {
    local db="$1"
    if run_sql "SELECT 1 FROM pg_database WHERE datname = '${db}'" | grep -q 1; then
        echo "==> 已存在，跳过：${db}"
    else
        run_sql "CREATE DATABASE ${db}" >/dev/null
        echo "==> 已创建：${db}"
    fi
}

create_db_if_missing "$DB_DEV"
create_db_if_missing "$DB_TEST"

echo "==> 数据库就绪：${DB_DEV} / ${DB_TEST}（${PGHOST}:${PGPORT}）"
