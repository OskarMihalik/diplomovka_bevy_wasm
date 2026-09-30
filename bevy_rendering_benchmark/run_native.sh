#!/usr/bin/env bash
# Runs the native benchmark once per model (one process each) and appends the results to
# bencmark_results/rendering_fps.csv and rendering_load.csv. Extra arguments are passed through:
#   bevy_rendering_benchmark/run_native.sh --gpu low
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p bevy_rendering_benchmark --release
for id in  13 14 15 16 17 18; do
    ./target/release/bevy_rendering_benchmark \
        --model "backend/assets/models/$id.glb" --out bencmark_results "$@"
done
