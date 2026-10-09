# Backend benchmark plan for the IEEE Access revision

This plan addresses Reviewer 1's item 10 (backend benchmarks: Axum/Tokio vs Express/Node, Cornucopia vs Kysely, pool sizes, sync vs async), item 12 (network latency and packet loss), and the "Pending: backend" parts of item 15 in `access-iee-26-Mihalik/changes_1.md`. Existing code and results stay as they are; everything new goes into new folders and new files.

---

## 1. Goals

| Goal | Reviewer item | Experiment |
|---|---|---|
| Compare the Axum/Tokio and Express/Node frameworks with no database | 10 | **A**: framework |
| Compare Cornucopia and Kysely as query layers, against each language's raw driver | 10 | **B**: query layer |
| Re-measure RQ3 with fair CPU allocation, repeated runs and percentiles | 10, 15 | **C**: full stack |
| Measure the effect of DB pool size | 10 | **C2**: pool sweep |
| Measure behavior under network latency and packet loss | 12 | **A** and **C**, profiles P0/P3/P5 |
| Measure CPU and RAM in Docker | 10 | all experiments, sampled from cgroups |
| Sync vs async | 10 | **D**: answered in the response letter, with an optional measurement |

**Decisions already made:**
- CPU: each configuration runs at **1 core** and at **4 cores**. Express at 4 cores uses `node:cluster` with 4 workers.
- Keep-alive is **matched** between the two frameworks.
- The endpoint list for A (section 5) is final.
- The statistics follow the rendering section: **5 runs, Welch's t-test, Δ = (A − B)/B**.

---

## 2. What stays unchanged

- `backend/`, `backend-express/`, `loadtest/` (including `*Report*.html` and `*.png`), `docker-compose.yml`, and every existing CSV in `bencmark_results/`.
- Experiment C needs only two small additions to the existing backends, controlled by env vars. When the env var isn't set, the backend behaves exactly as it does now:
  - `DB_POOL_SIZE` (default 10, as today)
  - `KEEP_ALIVE_TIMEOUT_MS`, for Express only (default: Node's default, as today)

  For Axum, `TOKIO_WORKER_THREADS` is read by Tokio itself, so no code change is needed. Express in cluster mode runs through a new wrapper, `framework_bench/express-cluster.js`, that loads `backend-express/dist/index.js`, so the existing code isn't modified for that.

---

## 3. New layout

Items marked (C) or (B) are not built yet.

```
framework_bench/
  PLAN.md                    this file
  README.md                  how to run experiment A
  bench.toml                 A: pinning, load, network profiles, configurations, cells
  express-cluster.js         node:cluster wrapper (A, and C at 4c)
  axum/                      A: minimal Axum server (workspace member)
  express/                   A: minimal Express 5 server (TS)
  docker/                    A: axum, express and attack images, each with its own .dockerignore
  runner/                    Rust crate (workspace member)
    src/bin/attack.rs        Goose attack, runs in the Goose container
    src/bin/runner/          host orchestrator: config, docker, netem, cgroup sampler, CSV
  scripts/
    smoke.sh                 A: identical responses check
    host_prepare.sh          turbo off, performance governor, AC check (sudo)
    db_snapshot.sh           (C) create seed DB template
  docker-compose.full.yml    (C) backend, backend-express, postgres, goose (edge_net + db_net)
  query_bench/               (B) rust/: tokio-postgres raw vs Cornucopia; node/: pg raw vs Kysely
  results/raw/<run_id>/      attack.json, attack.log, resources.csv, netem.txt, server.log (gitignored)

bencmark_results/
  framework_load.csv         A
  framework_load_pilot.csv   A, --pilot runs
  query_layer.csv            (B)
  backend_fullstack.csv      (C) + C2
  src/backend.rs             new loader + stats + table/figure generation
  paper/figures/be_*.tex     new figures
  out/tables/be_*.tex        new tables → copied to the paper's tables/
```

---

## 4. Shared infrastructure (used by A and C)

### 4.1 Host preparation (`sudo scripts/host_prepare.sh` before every batch)
- **Turbo off** (`intel_pstate/no_turbo = 1`), `performance` governor on every CPU, `performance` power profile, AC power, Legion power mode Performance, no other workloads. `--restore` switches back afterwards.
- Why turbo off: in the pilot with turbo on, the package reached 98–100 °C and throttled, so clocks varied between cells (2.0–4.9 GHz average). Absolute throughput is lower with turbo off, but the comparison is stable and repeatable.
- The runner refuses a full batch when turbo is on or the governor is not `performance` (override: `--allow-turbo`); pilots only warn. Every row records `turbo`, `governor`, `cpu_temp_max_c` and `cpu_freq_avg_mhz`.

### 4.2 Core pinning (`bench.toml`)
`lscpu -e` on the i7-14700HX: CPUs 0–15 are 8 P-cores with two threads each (CPU 2n and 2n+1 share core n; cores 4–5, CPUs 8–11, boost higher than the rest), CPUs 16–27 are 12 E-cores.

| Role | CPUs |
|---|---|
| Server, 1c | 2 (core 1; sibling 3 idle) |
| Server, 4c | 2, 4, 6, 12 (cores 1, 2, 3, 6; siblings idle; all the same max clock) |
| Postgres (C only) | 8, 10 |
| Goose | 8–11, 14–15, 16–27 (12 E-cores + the free P-cores 4, 5, 7; in C, 8 and 10 go to Postgres) |
| Host, Docker, runner | 0–1 |

With only the 12 E-cores, Goose was saturated (9.6–10.2 cores) by `axum-1c` on `/plaintext` and `/json` in the pilot; with 18 CPUs it used 7.5.

The server has a **2 GiB** memory limit (no swap) in every configuration. At 1 GiB, `express-4c /upload` was OOM-killed in the pilot (4 workers buffering 1 MiB bodies; it peaks at about 1.2 GiB).

### 4.3 Configurations
| id | Server | cpuset | Runtime |
|---|---|---|---|
| `axum-1c` | Axum | 2 | `TOKIO_WORKER_THREADS=1`, `BLOCKING_THREADS=1` |
| `axum-4c` | Axum | 2,4,6,12 | `TOKIO_WORKER_THREADS=4`, `BLOCKING_THREADS=4` |
| `express-1c` | Express | 2 | single process, `UV_THREADPOOL_SIZE=1` |
| `express-4c` | Express | 2,4,6,12 | `express-cluster.js`, 4 workers (round-robin), `UV_THREADPOOL_SIZE=4` |

Images: `rust:1-bookworm` build → `debian:bookworm-slim` for Axum; `node:24-bookworm-slim` (Node 24 LTS) for Express, so both run on glibc. The Axum crate is built standalone with the workspace `Cargo.lock`. Each row records the server image id.

**Blocking pools are matched:** `BLOCKING_THREADS` sets Tokio's `max_blocking_threads` to the same size as Node's `UV_THREADPOOL_SIZE`. Both pools run argon2 (`/cpu`) and file reads (`ServeFile` uses `tokio::fs`, `res.sendFile` uses libuv `fs`). With Tokio's default of 512 threads, all 256 users hashed at once (256 × 19 MiB ≈ 4.8 GiB) and Axum was OOM-killed in the pilot, while the bounded libuv pool was not. The old `backend/` uses the default pool, so this is an observation worth a sentence in the paper.

**Keep-alive is matched:** hyper doesn't close idle connections, so Express is set to the same with `server.keepAliveTimeout = 0` and `headersTimeout = 120 s`. The Goose client keeps connections alive. The Goose request timeout stays at the default 60 s, as in the paper.

**TCP_NODELAY is off in both (Nagle enabled):** `axum::serve` leaves it off, and the existing `backend/src/main.rs` does too. Node turns it on by default, so Express servers are created with `http.createServer({ noDelay: false }, app)`. In C, `backend-express` needs the same setting (env-gated, default unchanged) so both full-stack backends match.

**Response-level parity in A:** Express's automatic `ETag` on `res.send`/`res.json` is turned off (it hashes every response body, and Axum does no equivalent work), and so is the `X-Powered-By` header. Static files use the idiomatic path in both: `tower-http` `ServeFile` and Express `res.sendFile` (without ETag).

### 4.4 Network emulation (`runner/src/bin/runner/netem.rs`)
- Traffic goes **container to container** (Goose → server) on the `fwbench_edge` network, so no ports are published and there's no docker-proxy in the path.
- A sidecar (the attack image, which ships `iproute2`) joins each container's network namespace with `NET_ADMIN` and runs `tc qdisc add dev eth0 root netem limit 100000 delay <RTT/2> <jitter/√2> distribution normal loss <loss>`, on **both** the Goose and the server side. Two independent normal delays of σ/√2 give a round trip with the profile's σ; loss applies to each direction. netem's default queue of 1000 packets is raised, because it would otherwise drop packets on large responses. Jitter reorders packets, as on real networks.
- In C, Postgres is on a separate `db_net` network without netem, so database queries aren't delayed.

| Profile | RTT | Jitter (σ) | Loss per direction | Meaning |
|---|---|---|---|---|
| P0 | – | – | 0 | local baseline |
| P3 | 60 ms | 15 ms | 0.1% | typical WAN / 4G |
| P5 | 120 ms | 40 ms | 2% | poor mobile connection |

Verified before the pilot: with the P5 delay and loss on both containers, 50 pings gave a 63.8 ms average RTT, 4% ping loss (2% each way), and the qdisc drop counter matched.

**Checks for each measurement:**
- before: 20 pings from Goose to the server → `measured_rtt_ms`
- after: `tc -s qdisc` on both sides → `netem_drops_client`, `netem_drops_server`; `/proc/net/snmp` in the Goose container → `tcp_retrans`

### 4.5 Load generation and latency (`runner/src/bin/attack.rs`)
- Goose drives the virtual users (closed loop, no wait time), the ramp-up and one HTTP client per user. Goose's own metrics are **disabled**:
  - Goose stores response times in whole milliseconds (rounded further above 100 ms), which cannot separate endpoints that answer in well under 1 ms.
  - Goose stops the timer when the response headers arrive; the attack reads the full body (in chunks, without buffering it).
- The attack records every request into a lock-free log-linear histogram in microseconds (exact below 128 µs, ≤0.8% error above), plus counters for 2xx, other statuses, 5xx, transport errors and timeouts. Percentiles are capped at the recorded maximum.
- Only requests that **complete** inside the window [ramp + 1 s, ramp + 1 s + run) are recorded, so the window contains only full load; Goose keeps the users running 1 s past it.
- Output: `attack.json` with the window's Unix timestamps, counts, rps, MB/s, mean/p50/p90/p95/p99/p99.9/max, and the non-empty histogram buckets.

### 4.6 CPU and RAM (`runner/src/bin/runner/sampler.rs`)
- Every 250 ms, a host thread reads the cgroup v2 files of the server and the Goose container: `cpu.stat` (`usage_usec`), `memory.current`, `memory.stat` (`anon`, `file`, `inactive_file`), plus the `x86_pkg_temp` zone and `scaling_cur_freq` of the server's CPUs.
- **CPU** = Δusage / Δt, in cores used. **RAM** = `current − inactive_file` (working set, as in `docker stats`); this includes kernel socket buffers charged to the container, which is why `/file` at P5 showed ~200 MiB in both servers. `mem_anon_peak_mib` is the application heap.
- For Express in cluster mode, the cgroup covers the whole container, so the numbers include all workers plus the primary.
- **Every cell gets a cold server container.** This keeps cells independent (no heap carried over from the previous endpoint), and it makes the kernel's `memory.peak` a valid per-cell peak (it can only be reset by root).
- Phases: **idle** (5 s after the server answers) → warm-up (netem, ping, ramp) → **measurement** (attack window) → **cooldown** (10 s).
- Goose is also sampled. If it uses more than 80% of its 18 CPUs during the window, the row gets `client_saturated=true`.
- After the attack, the runner records the server container's state and the `oom_kill` count from `memory.events` (`server_state`, `oom_kills`). In the first pilot, three OOM-killed servers produced rows with 100% transport errors and zero CPU; those rows are now flagged and printed as warnings.

### 4.7 Runner (`runner/src/bin/runner/main.rs`)
```
start Goose container (E-cores, results/raw mounted at /out)
for rep in 1..=reps:
  for config in shuffle(configs):              # seeded, seed recorded
    for cell in shuffle(cells for this config):
      [C] restore DB from template
      start server container (cold), probe /plaintext until 200
      sample idle 5 s
      apply netem on both sides, ping 20×, read retrans counter
      docker exec Goose: attack run (ramp 10 s, window 60 s)
      cooldown 10 s, stop sampler
      read tc drops, retrans, memory.peak; clear netem; save logs; remove server
      append CSV row, write raw files
```
Options: `--build`, `--pilot` (1 repetition, 20 s windows, separate CSV), `--reps`, `--seed`, `--only <configs>`, `--cells <endpoint/profile,...>`, `--dry-run`, `--allow-turbo`. Rows already in the CSV with the same seed are skipped, so rerunning with `--seed` resumes an interrupted batch. A failed cell is reported and the batch continues.

---

## 5. Experiment A: framework without a database

**Servers:** `framework_bench/axum` and `framework_bench/express`, with identical endpoints. No middleware except JSON and raw-body parsing; no CORS, no logging.

| Endpoint | Measures |
|---|---|
| `GET /plaintext` | raw HTTP stack overhead |
| `GET /json` | serializing a small object |
| `POST /echo-json` (~2 KB) | JSON parse, validate and serialize |
| `GET /json-large` (~450 KB, 1,600 tag-like objects) | serializing large responses |
| `GET /cpu` (argon2id, same parameters and pepper as the backends: 19,000 KiB, t=2, p=1, 32 B) | CPU-bound work; `spawn_blocking` vs the libuv pool |
| `GET /delay?ms=20` | waiting on async I/O (concurrency model) |
| `GET /file` (`loadtest.glb`, 3.7 MiB, the model the old load test uploads) | streaming downloads |
| `POST /upload` (1 MiB `application/octet-stream`) | request-body handling |

**Smoke test (`smoke.sh`):** both servers must return identical status codes and bodies (JSON compared semantically) before any measurement starts. `/cpu` uses a fixed salt, so both must produce the byte-identical PHC string, which proves the argon2 parameters match.

**Cells:**

| Profile | 1c configs | 4c configs |
|---|---|---|
| P0 | all 8 endpoints | `/json-large`, `/cpu`, `/delay`, `/file`, `/upload` |
| P3 | `/json`, `/delay`, `/file` | same |
| P5 | `/json`, `/delay`, `/json-large` | same |

- `/plaintext`, `/json` and `/echo-json` at P0 run only at 1 core. At 4 cores the load generator is the bottleneck: in the pilot, Goose used 11.6 of its 12 E-cores while `axum-4c` used only 2.2 of its 4 cores. The 4c comparison uses the heavier endpoints, where the server is the bottleneck.
- At P5, `/file` is replaced by `/json-large`. A 3.7 MiB download takes 25–30 s at 2% loss per direction (TCP throughput collapses), so few downloads complete in the window, and the server is idle (0.02 cores) in both frameworks.
- That's 14 cells per 1c configuration and 11 per 4c configuration.

**Load:** 256 concurrent Goose users, the same for all four configurations.

**Time:** (2 × 14 + 2 × 11) cells × ~98 s × 5 repetitions ≈ **6.8 h**.

---

## 6. Experiment B: query layer, Cornucopia vs Kysely

**Why the raw drivers are included:** comparing Cornucopia (Rust) with Kysely (TypeScript) directly mixes two effects, the language/driver and the abstraction layer. Four variants keep them apart:

| Variant | Language | Layer |
|---|---|---|
| `rust-raw` | Rust | `tokio-postgres` + bb8, prepared statements |
| `rust-cornucopia` | Rust | Cornucopia-generated functions + bb8 |
| `node-raw` | Node | `pg` Pool, parameterized queries |
| `node-kysely` | Node | Kysely + `pg` Pool |

The comparisons are then:
- **Abstraction overhead:** Cornucopia vs raw, Kysely vs raw
- **Stack:** Cornucopia vs Kysely
- **Driver:** raw vs raw

**Queries:** the same SQL the apps run, taken from the existing `backend/` queries and the Kysely code:
1. `select user by email` (the login lookup)
2. `insert tag`
3. `select tags by project` (~50 rows)
4. `insert project + delete project` (cascade)

**Harness:** an in-process loop with C concurrent tasks (C = 10 and C = 50, the second exceeding the pool to show queuing), 30 s per query, 5 s warm-up, pool size 10. The client is pinned to 1 P-core (fair to Node's single JS thread), and Postgres to its own 2 P-cores. The DB is restored from the template before each repetition, and the variant order is randomized.

**Metrics:** ops/s, p50/p95/p99 latency, client CPU-seconds per 1k ops, client RSS.

**Time:** 4 variants × 4 queries × 2 concurrency levels × 35 s × 5 repetitions ≈ **1.3 h**.

---

## 7. Experiment C: full stack (re-measuring RQ3)

**Servers:** the existing `backend/` and `backend-express/`, unchanged except for the env vars in section 2.

**Database:**
- Postgres 14 (as in the paper) on `db_net`, pinned to 2 P-cores, with the same configuration for both backends.
- `db_snapshot.sh` creates a `bevy_seed` database (schema plus the test user).
- Before every run: `DROP DATABASE bevy; CREATE DATABASE bevy TEMPLATE bevy_seed`. This is fast and gives every run an identical starting state.

**Scenarios** (copied from the old `loadtest`, to keep continuity with the paper):
- `tag`: 1000 users; login → create project → upload model → repeatedly create tags → delete project
- `login`: 100 users, repeated login

**Cells:** 4 configurations × 2 scenarios × profiles P0/P3/P5.

**Time:** 4 × 2 × 3 × ~85 s × 5 repetitions ≈ **2.8 h**.

### C2: pool-size sweep
- `axum-4c` and `express-4c`, `tag` scenario, P0, `DB_POOL_SIZE` ∈ {5, 10, 20, 50}
- **Time:** 2 × 4 × ~85 s × 5 repetitions ≈ **1 h**

---

## 8. D: sync vs async (optional)

**Recommendation:** answer this in the response letter. Both evaluated stacks are asynchronous (Tokio and the Node event loop), and the incorrect phrase "synchronous stacks" was already removed under item 4.

If the supervisor wants data anyway, add an `axum-sync-db` variant: the same backend, but using the blocking `postgres` crate inside `spawn_blocking`. Run it only on the `tag` scenario, P0, 4c. That's about 10 minutes of runs and one extra row in the C2 table.

---

## 9. Output formats

All three CSVs share these metadata columns:
`run_id,date,host,os,kernel,cpu_model,turbo,governor,docker_version,server_image,seed,rep,config,framework,cores,cpuset,profile,rtt_ms,jitter_ms,loss_pct,measured_rtt_ms,netem_drops_client,netem_drops_server,tcp_retrans,client_saturated,goose_cpu_avg_cores,cpu_temp_max_c,cpu_freq_avg_mhz`

**`framework_load.csv` (A)** adds (latencies in ms, memory in MiB):
`server_state,oom_kills,endpoint,users,duration_s,requests,rps,mb_per_s,err_pct,status_other,status_5xx,transport_errors,timeouts,lat_mean,lat_p50,lat_p90,lat_p95,lat_p99,lat_p999,lat_max,cpu_idle_cores,cpu_avg_cores,cpu_p95_cores,cpu_util_pct,cpu_s_per_1k_req,mem_idle_mib,mem_avg_mib,mem_p95_mib,mem_max_mib,mem_peak_mib,mem_after_mib,mem_anon_peak_mib`

- `mem_*_mib` (except the two below) are the sampled working set; `mem_max_mib` is its maximum in the window.
- `mem_peak_mib` is the kernel's `memory.peak` over the container's life (one cell), which also counts page cache.
- `mem_anon_peak_mib` is the maximum anonymous memory (application heap) in the window.

**`backend_fullstack.csv` (C, C2)** has the same columns as A, plus:
`scenario,pool_size,status_0,status_5xx,db_cpu_avg_cores,db_mem_peak_mib`

Recording `status_0` and `status_5xx` directly fixes the inconsistent failure counts currently in the paper.

**`query_layer.csv` (B)** adds:
`variant,language,layer,query,concurrency,pool_size,ops,ops_per_s,lat_p50,lat_p95,lat_p99,cpu_s_per_1k_ops,rss_peak_mib`

Raw files go to `framework_bench/results/raw/<run_id>/` (gitignored): the attack summary with the latency histogram, the attack and server logs, the resource time series, and the ping and `tc -s` output.

---

## 10. Statistics (`bencmark_results/src/backend.rs` + `stats.rs`)

- **The unit is one run** (n = 5 per cell). Requests aren't independent samples.
- **Per cell:** mean ± SD and median.
- **Comparisons:**
  - A and C: Axum vs Express at 1c and at 4c
  - B: the variant pairs from section 6
  - C2: pool sizes against pool 10
- **Effect and test:**
  - Δ = (A − B)/B, with a 95% CI from Welch's interval
  - **Welch's t-test**, the same as the rendering section. Wilcoxon isn't used, because with 5 pairs it can never reach p < 0.05.
  - **Holm correction** within each experiment
  - A result is reported as a difference only when p_adj < 0.05 **and** |Δ| ≥ 5%. Otherwise it's "no meaningful difference".
- **Scaling efficiency:** `rps_4c / (4 × rps_1c)` for each framework.
- **Network effect:** the change in p99 and RPS from P0 to P3 and P5, for each configuration.

---

## 11. Tables and figures for the paper

Everything is generated by `bencmark_results`. Nothing in these tables is edited by hand.

| File | Contents |
|---|---|
| `tables/be_fullstack.tex` | C: RPS, p50, p99, errors (0/5xx), CPU, peak RAM; 4 configurations × 2 scenarios (P0). The main table for RQ3 |
| `tables/be_fullstack_ttest.tex` | C: Δ and p for Axum vs Express at 1c and 4c, plus scaling efficiency |
| `tables/be_framework.tex` | A: RPS and p99 per endpoint, 4 configurations (P0) |
| `tables/be_query.tex` | B: ops/s and p99, 4 variants × 4 queries, with Δ |
| `tables/be_pool.tex` | C2: RPS and p99 for pool sizes 5/10/20/50 |
| `tables/be_network.tex` | A and C: p99 and RPS for P0/P3/P5 |
| `figures/be_scaling.tex` | 1c vs 4c RPS (A and C) |
| `figures/be_network.tex` | p99 vs profile, per configuration |
| `figures/be_resources.tex` | CPU-s per 1k requests and peak RAM |
| `figures/be_timeseries.tex` | CPU and RAM over time for one representative run |

---

## 12. Changes to the paper (`access.tex`)

1. **Server Technologies section:** state that Express runs on Node.js (event loop plus libuv pool) and that the 4c configuration uses cluster mode.
2. **Backend Concurrency section:** rewrite the methodology:
   - the 3 levels (A/B/C), 1c/4c, pinning, keep-alive, netem, cgroups
   - 5 runs, Welch's t-test, Holm correction
   - new test configuration (Kubuntu, kernel 7.0), listed separately from the Pop!_OS spec
3. **Replace** the 4 Goose screenshots, 4 hand-copied tables and 4 CPU/RAM PNGs with the generated tables and figures from section 11. The old files stay in the repo and in `assets/`.
4. **Discussion VI.A:** the explanation "multi-threaded Tokio vs single-threaded Node" is now tested directly by the 1c/4c comparison. Rewrite it based on the measured result. Separate the contributions of the framework (A), the query layer (B) and the network (P3/P5).
5. **RQ3:** keep the wording. The answer and the RQ summary table use the new numbers and state the framework/query/full-stack breakdown.
6. **Keep these consistent everywhere:** the new numbers replace "1,453.14 RPS / about 2×" in the abstract, contributions, Discussion VI.A, VI.C point 2, the RQ table, the RQ3 answer and the conclusion. Update the paper's `CLAUDE.md` (Key facts) at the same time.
7. **Limitations:** remove "To eliminate network variability…". Add that the network was emulated (netem) rather than measured on a real WAN, and that everything ran on a single machine.
8. **`changes_1.md`:** set items 10, 12 and 15 to ✅ and close the open question about the backend tables.

---

## 13. Order of work

| # | Step | Result | Estimate |
|---|---|---|---|
| 1 | ✅ A servers + `smoke.sh` | 2 servers returning identical responses | done |
| 2 | ✅ Runner: Docker, pinning, cgroup sampler, netem, CSV | infrastructure; 6 test cells run end to end | done |
| 3 | ✅ A pilot (`--pilot`, turbo off, 1 repetition, 20 s): 50 cells in 46 min; found 3 OOM kills and Goose saturation, both fixed and the affected cells rerun. Still open: comparison with `docker stats` | harness validated | done |
| 4 | **A full run** | `framework_load.csv` | overnight, ~6.8 h |
| 5 | C: env vars in the backends, DB template, `docker-compose.full.yml`, fullstack attack | ready for C | 1 day |
| 6 | C pilot, then **C + C2 full run** | `backend_fullstack.csv` | overnight, ~3.8 h |
| 7 | B: Rust + Node harness, Cornucopia and Kysely on the same SQL | ready for B | 1 day |
| 8 | **B run** | `query_layer.csv` | ~1.3 h |
| 9 | `bencmark_results`: loaders, statistics, tables, figures | `be_*.tex` | 1 day |
| 10 | Paper + `changes_1.md` + response letter (items 10, 12) | revision | 1–2 days |

Total compute time is about 12 h (two nights). Total work is about 8–10 days.

---

## 14. Risks and how they're handled

| Risk | Mitigation |
|---|---|
| Laptop thermal throttling | turbo off and the performance governor (enforced by the runner); randomized order; temperature and frequency recorded per row |
| Goose becoming the bottleneck | runs on 18 CPUs (12 E-cores + 6 free P-core threads); Goose metrics disabled; `client_saturated` flag; light endpoints not run at 4c (section 5) |
| Server dies under load (OOM) | 2 GiB limit; matched blocking pools; `server_state` and `oom_kills` recorded per row and printed as warnings |
| P-core vs E-core differences | the server always runs on P-cores; the layout is recorded |
| Netem not actually applied | RTT probe and drop counters stored for every run |
| DB state drifting between runs | restored from the template before every run |
| Express 4c cluster balancing | Node's default round-robin scheduling (on Linux); recorded in the README |
| Cornucopia vs Kysely mixing language and layer | raw-driver baselines in both languages |
| The results weaken the "about 2×" claim | acceptable: the paper already frames claims modestly, and reviewers prefer defensible numbers |
