# 菜芦通用后台系统 —— 后端镜像（容器内多阶段编译）
#
# 说明：
#   * 后端通过 crates.io 依赖 Alun 框架发布版（alun 0.4.0），构建时由 cargo 自动拉取，
#     无需本地框架源码，也不再需要 scripts/deploy.sh 暂存本地框架源码的步骤；
#   * 编译在容器内完成，产物为 Linux 二进制，与宿主机（macOS/Windows）无关。
#
# 构建（构建上下文须为 cailu 项目根，含 backend/）：
#   cd cailu/cailu && docker build -f docker/backend/Dockerfile -t cailu-backend .

# ---- 构建阶段 ----
# rust 官方镜像按大版本打标签（rust:1 / rust:1-bookworm），无 1.94 小版本标签
FROM rust:1-bookworm AS builder

# ring / openssl 等原生依赖需要的编译工具
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY backend/ /app/backend/

RUN cd /app/backend && cargo build --release

# ---- 运行阶段 ----
FROM debian:bookworm-slim

ENV ALUN_PROFILE=prod

COPY --from=builder /app/backend/target/release/cailu /usr/local/bin/cailu

# 运行时需要的配置与迁移（App::new() 读取 config/，迁移 runner 读取 migrations/）
COPY backend/config/ /app/config/
COPY backend/migrations/ /app/migrations/

# 运行时目录：上传 / 下载 / 日志（挂载卷时以 -v 覆盖）
RUN mkdir -p /app/uploads /app/downloads /app/logs
WORKDIR /app

# 生产监听地址由 ALUN_SERVER_LISTEN 覆盖（默认 0.0.0.0:8023）
ENV ALUN_SERVER_LISTEN=0.0.0.0:8023

EXPOSE 8023

CMD ["cailu"]
