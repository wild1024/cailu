#!/usr/bin/env bash
# 提交前检查：后端 fmt / clippy / check + 前端 typecheck
#
# 用法：bash scripts/check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> 后端：cargo fmt --check"
(cd "$ROOT/backend" && cargo fmt -- --check)

echo "==> 后端：cargo clippy -- -D warnings"
(cd "$ROOT/backend" && cargo clippy --all-targets -- -D warnings)

echo "==> 后端：cargo check"
(cd "$ROOT/backend" && cargo check)

echo "==> 前端：pnpm typecheck"
(cd "$ROOT/frontend" && pnpm typecheck)

echo "==> 全部检查通过"
