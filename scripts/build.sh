#!/usr/bin/env bash
# 构建产物：后端 release 二进制 + 前端 dist
#
# 用法：bash scripts/build.sh
# 产物：
#   backend/target/release/cailu   后端 Linux/本机 release 二进制（本机运行 / CI 用；生产镜像由 docker 内编译，不依赖本产物）
#   frontend/dist/                 前端构建产物（供 docker/frontend/Dockerfile 与 nginx 托管）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> 后端：cargo build --release"
(cd "$ROOT/backend" && cargo build --release)

echo "==> 前端：pnpm build（含类型检查）"
(cd "$ROOT/frontend" && pnpm build)

echo "==> 构建完成"
echo "    后端二进制：$ROOT/backend/target/release/cailu"
echo "    前端产物 ：$ROOT/frontend/dist/"
