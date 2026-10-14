# Experiment B: Kysely client. Build from the repository root:
#   docker build -f framework_bench/docker/qbench-node.Dockerfile -t qbench-node .
# Same Node image as the Express server; the Kysely types are copied from backend-express.
FROM node:24-bookworm-slim AS build
WORKDIR /app
COPY framework_bench/query_bench/node/package.json framework_bench/query_bench/node/package-lock.json ./
RUN npm ci
COPY framework_bench/query_bench/node/tsconfig.json ./
COPY framework_bench/query_bench/node/src/index.ts ./src/
COPY backend-express/src/db/types.ts ./src/db/
RUN npx tsc && npm prune --omit=dev

FROM node:24-bookworm-slim
WORKDIR /app
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist
ENV NODE_ENV=production
ENTRYPOINT ["node", "dist/index.js"]
