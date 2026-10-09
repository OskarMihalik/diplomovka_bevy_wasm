# Experiment A: Axum server. Build from the repository root:
#   docker build -f framework_bench/docker/axum.Dockerfile -t fwbench-axum .
# The crate is built standalone with the workspace Cargo.lock, so dependency versions match.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.lock ./
COPY framework_bench/axum/ ./
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /src/target/release/framework_bench_axum /usr/local/bin/framework_bench_axum
COPY loadtest/src/loadtest.glb /data/loadtest.glb
ENV FILE_PATH=/data/loadtest.glb PORT=8080
CMD ["framework_bench_axum"]
