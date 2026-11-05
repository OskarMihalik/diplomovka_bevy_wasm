cargo run --release -- -t 60s -s 5 -u 1000

LOADTEST_SCENARIO=login cargo run --release -- -t 60s -s 5 -u 100

The first command runs the tag creation scenario, the second one the login scenario (its files
get the suffix `-login`). Turn turbo off first: `sudo framework_bench/scripts/host_prepare.sh`.

The tag creation scenario needs the index `Tag_project_id_created_at_idx` (migration
`20261027120000_tag_project_created_at_index`), the test checks for it and stops without it.
Apply it with `npx prisma migrate deploy` in `prisma/`.

The test starts `postgres`, `backend` and `backend-express` with docker compose, runs the attack
against Axum and then against Express, and writes `axumReport.html`, `expressReport.html`,
`cpu_usage.png`, `mem_usage.png` and `container_stats.csv` (CPU and memory of both backends and
of PostgreSQL, sampled about every second, with the backend under attack) into the current
directory.

Repeated runs for the paper (5 per scenario):

```
for r in 1 2 3 4 5; do LOADTEST_RUN=$r cargo run --release -- -t 60s -s 5 -u 1000; done
for r in 1 2 3 4 5; do LOADTEST_SCENARIO=login LOADTEST_RUN=$r cargo run --release -- -t 60s -s 5 -u 100; done
```

With `LOADTEST_RUN=<n>` the files get the suffix `-r<n>` (`axumReport-r2.html`,
`expressReport-login-r2.html`, ...) and even runs attack Express first, so neither backend is
always tested second.

- Every goose user has its own account (`goose-<n>@loadtest.local`). The accounts are created on
  the first run and stay in the database, so the first run on a new database also contains
  `/register` requests.
- A request counts as failed when the backend answers with an `Err` body, not only on a
  non-2xx status code.
- Both backends get the same resources from `BACKEND_THREADS` (default 28, one per logical CPU of
  the test machine; set it with `BACKEND_THREADS=<n> cargo run ...`), see `docker-compose.yml`:

  | | `backend` (Axum) | `backend-express` |
  | --- | --- | --- |
  | CPU workers | `TOKIO_WORKER_THREADS` Tokio workers | `WORKERS` cluster processes (`src/cluster.ts`) |
  | argon2 / file threads | `BLOCKING_THREADS` | `UV_THREADPOOL_SIZE=1` per process |
  | database connections | `DB_POOL_SIZE` | `DB_POOL_SIZE=1` per process |

  So each backend runs `BACKEND_THREADS` workers, computes at most `BACKEND_THREADS` hashes at
  the same time and holds `BACKEND_THREADS` database connections. The reports `-r1` to `-r5`
  in this directory are from this setup (29 Oct 2026). `old-asymmetric/` keeps the earlier
  runs: a single Express process with `UV_THREADPOOL_SIZE=28`, and Axum on the default Tokio
  runtime with a pool of 10.
- Before each attack the projects of earlier load tests (`goose-project-*`) are deleted and the
  tables are vacuumed, so both backends start from the same database state.
