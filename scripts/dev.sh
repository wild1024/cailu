#!/usr/bin/env bash
# 开发环境一键启动：依赖容器（PostgreSQL + Redis）→ 后端 → 前端
#
# 用法：bash scripts/dev.sh
# 停止：Ctrl+C（前端 / 后端进程随脚本退出一并终止）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/docker/docker-compose.yml"

echo "==> [1/3] 启动开发依赖（PostgreSQL + Redis）"
docker compose -f "$COMPOSE_FILE" up -d

# 等待数据库就绪（最多 30s）
echo "==> 等待数据库健康..."
for _ in $(seq 1 30); do
  status="$(docker inspect -f '{{.State.Health.Status}}' alun-db-postgres-1 2>/dev/null || true)"
  [ "$status" = "healthy" ] && break
  sleep 1
done

# 前端 / 后端两个前台进程，退出时一并清理
trap 'echo "==> 停止开发服务"; kill 0 2>/dev/null || true' EXIT INT TERM

echo "==> [2/3] 启动后端（:8023，自动迁移）"
(cd "$ROOT/backend" && cargo run) &

echo "==> [3/3] 启动前端（:5173，代理 /api、/health → :8023）"
(cd "$ROOT/frontend" && pnpm dev) &

wait
