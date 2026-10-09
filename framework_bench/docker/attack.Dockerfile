# Goose load generator plus the tools the runner uses inside the container (ping, tc).
# Build from the repository root:
#   docker build -f framework_bench/docker/attack.Dockerfile -t fwbench-attack .
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.lock ./
COPY framework_bench/runner/ ./
RUN cargo build --release --bin attack

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates iproute2 iputils-ping \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/attack /usr/local/bin/attack
CMD ["sleep", "infinity"]
