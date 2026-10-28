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
- `backend-express` runs with `UV_THREADPOOL_SIZE=28` (one thread per logical CPU of the test
  machine) instead of the default 4, so that it can compute the Argon2id hashes of the login on
  as many CPUs as the Tokio blocking pool of `backend`.
- Before each attack the projects of earlier load tests (`goose-project-*`) are deleted and the
  tables are vacuumed, so both backends start from the same database state.
