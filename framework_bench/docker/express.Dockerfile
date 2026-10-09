# Experiment A: Express server, single process or node:cluster (WORKERS). Build from the repository root:
#   docker build -f framework_bench/docker/express.Dockerfile -t fwbench-express .
# Same Debian base as the Axum image, so both use glibc.
FROM node:24-bookworm-slim AS build
WORKDIR /app
COPY framework_bench/express/package.json framework_bench/express/package-lock.json ./
RUN npm ci
COPY framework_bench/express/tsconfig.json ./
COPY framework_bench/express/src ./src
RUN npm run build && npm prune --omit=dev

FROM node:24-bookworm-slim
WORKDIR /app
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist
COPY framework_bench/express-cluster.js ./
COPY loadtest/src/loadtest.glb /data/loadtest.glb
ENV NODE_ENV=production FILE_PATH=/data/loadtest.glb PORT=8080
CMD ["node", "express-cluster.js", "dist/index.js"]
