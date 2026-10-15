# 菜芦通用后台系统 —— 前端镜像（构建产物 + nginx 托管）
#
# 构建（构建上下文须为 cailu 项目根）：
#   cd cailu/cailu && docker build -f docker/frontend/Dockerfile -t cailu-frontend .
# 运行：见 docker-compose.prod.yml（nginx 托管 dist 并反代 /api、/health 到后端）

# ---- 构建阶段：安装依赖并产出 dist ----
FROM node:22-alpine AS build

WORKDIR /app
# 启用 pnpm（版本由 package.json 的 packageManager 固定）
RUN corepack enable

# 先只拷贝清单文件以复用依赖层缓存
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

# 拷贝源码并构建（含 .env.production 环境变量）
COPY frontend/ ./
RUN pnpm build

# ---- 运行阶段：nginx 托管 ----
FROM nginx:1.27-alpine

COPY docker/frontend/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80
