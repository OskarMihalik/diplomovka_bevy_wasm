cargo run --release -- -t 60s -s 5 -u 1000

LOADTEST_SCENARIO=login cargo run --release -- -t 60s -s 5 -u 100

The first command runs the tag creation scenario, the second one the login scenario (its files
get the suffix `-login`). Turn turbo off first: `sudo framework_bench/scripts/host_prepare.sh`.

The test starts `postgres`, `backend` and `backend-express` with docker compose, runs the attack
against Axum and then against Express, and writes `axumReport.html`, `expressReport.html`,
`cpu_usage.png` and `mem_usage.png` into the current directory.

- Every goose user has its own account (`goose-<n>@loadtest.local`). The accounts are created on
  the first run and stay in the database, so the first run on a new database also contains
  `/register` requests.
- A request counts as failed when the backend answers with an `Err` body, not only on a
  non-2xx status code.
- Before each attack the projects of earlier load tests (`goose-project-*`) are deleted and the
  tables are vacuumed, so both backends start from the same database state.
