# Framework benchmark: Axum vs Express

Experiment A from [PLAN.md](PLAN.md): Axum/Tokio and Express/Node, each at 1 and 4 cores, with no database, under three network profiles. Load comes from Goose; CPU and memory come from the containers' cgroups. Run everything from the repository root.

## 1. Check the servers locally

```sh
cargo build --release -p framework_bench_axum
(cd framework_bench/express && npm ci && npm run build)
PORT=18081 ./target/release/framework_bench_axum &
PORT=18082 node framework_bench/express-cluster.js framework_bench/express/dist/index.js &
framework_bench/scripts/smoke.sh http://localhost:18081 http://localhost:18082
```

The smoke test must print `all checks passed`. To check the 4-core setup, start the servers with `TOKIO_WORKER_THREADS=4` and `WORKERS=4 UV_THREADPOOL_SIZE=4`.

## 2. Prepare the host

```sh
sudo framework_bench/scripts/host_prepare.sh   # turbo off, performance governor
```

Plug in the charger, set the Legion power mode to Performance (Fn+Q), and close other applications. Afterwards, `sudo framework_bench/scripts/host_prepare.sh --restore` switches turbo back on.

## 3. Run

```sh
cargo build --release -p framework_bench_runner
./target/release/runner --build --pilot          # build the images, then a ~50 min pilot
./target/release/runner --dry-run                # show the order and the estimated time
./target/release/runner                          # full batch, about 6.8 h
```

- The runner prints its seed at the start. If a batch is interrupted, run it again with `--seed <seed>`; it skips rows already in the CSV.
- `--only axum-1c,express-1c` and `--cells json/P0,file/P3` limit a run to some configurations or cells.
- A full batch refuses to start with turbo on (`--allow-turbo` overrides this); `--pilot` only warns.

## Output

- `bencmark_results/framework_load.csv`: one row per configuration, cell and repetition (`--pilot` writes `framework_load_pilot.csv`). PLAN.md section 9 describes the columns.
- `framework_bench/results/raw/<run_id>/`: `attack.json` (summary and latency histogram), `resources.csv` (CPU and memory every 250 ms, by phase), `netem.txt` (ping and `tc -s` output), `attack.log`, `server.log`.

## Setup decisions

These are explained in PLAN.md sections 4 and 5:

- **Containers:** each cell gets a cold server container.
- **CPU pinning:** the server runs on P-cores 2 / 2,4,6,12; Goose runs on CPUs 8–11, 14–15 and the E-cores 16–27.
- **Memory:** 2 GiB per server. Axum's blocking pool is capped at the same size as Node's libuv pool (`BLOCKING_THREADS` = `UV_THREADPOOL_SIZE`).
- **Dead servers:** every row records `server_state` and `oom_kills`, and the runner prints a warning when a server died.
- **Server settings:** keep-alive and `TCP_NODELAY` (off) are matched in both servers, and Express's ETag is off.
- **Latency:** recorded by the attack itself, in microseconds, over the full response body. Goose's millisecond metrics are disabled.
- **netem:** applied on both sides, with RTT/2 and jitter/√2 per side and the loss rate per direction.
- **Turbo:** must be off for a full batch, because with turbo on the laptop throttles at 100 °C.
- **Excluded cells:** `/plaintext`, `/json` and `/echo-json` don't run at 4 cores, because Goose saturates first. At P5, `/file` is replaced by `/json-large`.
