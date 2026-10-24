#!/usr/bin/env bash
# 生产部署：构建前端 dist → 构建镜像 → 拉起完整编排（postgres + redis + backend + frontend）
#
# 用法：bash scripts/deploy.sh
# 前置：
#   * docker 可用；
#   * 部署前导出生产敏感环境变量（见 docker/docker-compose.prod.yml 顶部说明）：
#       ALUN_DATABASE_PASSWORD / ALUN_MIDDLEWARE_AUTH_JWT_SECRET（必填）
#       ALUN_ADMIN_PASSWORD（可选）
#   * 后端通过 crates.io 依赖 Alun 框架 0.4.2，构建时由 cargo 自动拉取，无需本地框架源码。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> [1/2] 构建前端 dist（后端镜像在容器内编译）"
bash "$ROOT/scripts/build.sh"

echo "==> [2/2] 构建镜像并启动生产编排"
docker compose -f "$ROOT/docker/docker-compose.prod.yml" up -d --build

echo "==> 部署完成：前端 http://<host>/（nginx :80，反代后端 :8023）"
