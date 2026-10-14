# Experiment B: Cornucopia vs Kysely

The database layer of both backends on its own, with no HTTP. See `../PLAN.md`, section 6.

```bash
sudo framework_bench/scripts/host_prepare.sh          # turbo off, performance governor
framework_bench/query_bench/run.sh --build --smoke    # build both images, check that they agree
framework_bench/query_bench/run.sh --pg-cpus 4,6,8,10 # 5 runs per variant → bencmark_results/query_layer.csv
sudo framework_bench/scripts/host_prepare.sh --restore
```

`run.sh` starts `postgres:14-alpine` (`qbench-pg`, network `qbench_net`), applies the Prisma migrations
and `seed.sql` to `qbench_seed`, and copies it into a fresh `qbench` before every run. Each run is one
client container on CPU 2 that prints one CSV row.

| Variant | Code | Image |
|---|---|---|
| `cornucopia` | `rust/` (the `model` crate + bb8, as in `backend/`) | `qbench-rust` |
| `kysely` | `node/` (Kysely + `pg.Pool`, as in `backend-express/`) | `qbench-node` |

Scenario per iteration: `user_by_email` → `tags_by_project` (100 rows) → `insert_tag` →
`project_tx` (project + admin membership in one transaction, then delete). Differences from the
backends' code, so both send the same SQL:

- the Kysely login lookup selects the 5 columns Cornucopia selects, not `selectAll()`
- the Kysely inserts leave `updated_at` to the column default (the controllers set `new Date()`)

Options: `--reps N` (default 5), `--csv PATH` (e.g. a pilot file), `--pg-cpus LIST` (default `8,10`),
`--allow-turbo`. Local builds: `SKIP_DB_BUILD=1 cargo build --release -p qbench`, and
`npm ci && npm run build` in `node/` (copies the Kysely types from backend-express).
