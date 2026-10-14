# Experiment B: Cornucopia client. Build from the repository root:
#   docker build -f framework_bench/docker/qbench-rust.Dockerfile -t qbench-rust .
# Built standalone with the workspace Cargo.lock, as the Axum image. SKIP_DB_BUILD makes `model`
# use its committed cornucopia.rs instead of regenerating it from a live database.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY model/ ./model/
COPY framework_bench/query_bench/rust/ ./framework_bench/query_bench/rust/
COPY Cargo.lock ./framework_bench/query_bench/rust/
ENV SKIP_DB_BUILD=1
RUN cd framework_bench/query_bench/rust && cargo build --release

FROM debian:bookworm-slim
COPY --from=build /src/framework_bench/query_bench/rust/target/release/qbench /usr/local/bin/qbench
ENTRYPOINT ["qbench"]
