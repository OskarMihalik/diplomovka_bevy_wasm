#!/usr/bin/env bash
# Experiment B (PLAN.md, section 6): Cornucopia vs Kysely, one database scenario, no HTTP.
#
#   framework_bench/query_bench/run.sh [--build] [--smoke] [--reps N] [--csv PATH] [--pg-cpus LIST] [--allow-turbo]
#
# Starts Postgres 14, builds the seed DB (Prisma migrations + seed.sql), checks that both clients
# return the same checksum (--smoke stops there), then runs N repetitions of each variant in
# alternating order. Every run starts from a fresh copy of the seed DB and appends one row to
# bencmark_results/query_layer.csv. Run `sudo framework_bench/scripts/host_prepare.sh` first.
set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT=$PWD

BUILD=0 SMOKE_ONLY=0 REPS=5 ALLOW_TURBO=0 CSV=$ROOT/bencmark_results/query_layer.csv
PG_CPUS=8,10  # as Postgres in C; written to the CSV with spaces (PLAN.md, 6.8)
while [[ $# -gt 0 ]]; do
  case $1 in
    --build) BUILD=1 ;;
    --smoke) SMOKE_ONLY=1 ;;
    --reps) REPS=$2; shift ;;
    --csv) CSV=$(realpath -m "$2"); shift ;;
    --pg-cpus) PG_CPUS=$2; shift ;;
    --allow-turbo) ALLOW_TURBO=1 ;;
    *) echo "usage: $0 [--build] [--smoke] [--reps N] [--csv PATH] [--pg-cpus LIST] [--allow-turbo]" >&2; exit 2 ;;
  esac
  shift
done

NETWORK=qbench_net
PG=qbench-pg
CLIENT_CPUS=2     # one P-core, as the 1c servers
WORKERS=10 WARMUP=100 ITERATIONS=1000

declare -A IMAGE=([cornucopia]=qbench-rust [kysely]=qbench-node)

# --- host state -------------------------------------------------------------------------------
turbo=unknown
if [[ -r /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
  turbo=$([[ $(cat /sys/devices/system/cpu/intel_pstate/no_turbo) == 1 ]] && echo off || echo on)
fi
if [[ $SMOKE_ONLY == 0 && $turbo != off && $ALLOW_TURBO == 0 ]]; then
  echo "turbo is $turbo: run 'sudo framework_bench/scripts/host_prepare.sh' first (or --allow-turbo)" >&2
  exit 1
fi
host=$(hostname)
cpu_model=$(lscpu | sed -n 's/^Model name: *//p' | head -1 | tr -d ',')

# --- images -----------------------------------------------------------------------------------
for variant in cornucopia kysely; do
  image=${IMAGE[$variant]}
  if [[ $BUILD == 1 ]] || ! docker image inspect "$image" > /dev/null 2>&1; then
    echo "building $image"
    docker build -q -f "framework_bench/docker/$image.Dockerfile" -t "$image" . > /dev/null
  fi
done

# --- Postgres and the seed DB -----------------------------------------------------------------
docker network inspect "$NETWORK" > /dev/null 2>&1 || docker network create "$NETWORK" > /dev/null
docker rm -f "$PG" > /dev/null 2>&1 || true
trap 'docker rm -f "$PG" > /dev/null 2>&1 || true' EXIT
# synchronous_commit=off: otherwise the inserts mostly measure the NVMe fsync (PLAN.md, 6.5)
docker run -d --name "$PG" --network "$NETWORK" --cpuset-cpus "$PG_CPUS" --memory 2g \
  -e POSTGRES_PASSWORD=postgres postgres:14-alpine \
  postgres -c synchronous_commit=off -c shared_buffers=256MB -c max_connections=100 > /dev/null
# TCP only comes up after the image's init phase (which listens on the socket only)
for _ in $(seq 60); do
  docker exec "$PG" pg_isready -h 127.0.0.1 -U postgres > /dev/null 2>&1 && break
  sleep 1
done

psql() { docker exec -i "$PG" psql -U postgres -v ON_ERROR_STOP=1 -q "$@"; }
psql -c 'CREATE DATABASE qbench_seed'
for migration in $(ls prisma/prisma/migrations | grep -v '\.toml$' | sort); do
  psql -d qbench_seed < "prisma/prisma/migrations/$migration/migration.sql"
done
psql -d qbench_seed < framework_bench/query_bench/seed.sql > /dev/null

reset_db() {
  psql -c 'DROP DATABASE IF EXISTS qbench WITH (FORCE)' 2> /dev/null
  psql -c 'CREATE DATABASE qbench TEMPLATE qbench_seed'
}

client() {
  local variant=$1; shift
  docker run --rm --network "$NETWORK" --cpuset-cpus "$CLIENT_CPUS" --memory 1g \
    -e TOKIO_WORKER_THREADS=1 "${IMAGE[$variant]}" "$@"
}

# --- smoke: same checksum, rows and table counts --------------------------------------------
declare -A smoke
for variant in cornucopia kysely; do
  reset_db
  out=$(client "$variant" --workers 1 --warmup 0 --iterations 1 --smoke)
  counts=$(psql -d qbench -tAc 'SELECT (SELECT count(*) FROM "Tag"), (SELECT count(*) FROM "Project"), (SELECT count(*) FROM "ProjectUser")')
  smoke[$variant]="checksum/rows $out, Tag/Project/ProjectUser $counts"
  echo "smoke $variant: ${smoke[$variant]}"
done
if [[ ${smoke[cornucopia]} != "${smoke[kysely]}" ]]; then
  echo "smoke failed: the variants differ" >&2
  exit 1
fi
[[ $SMOKE_ONLY == 1 ]] && exit 0

# --- runs -------------------------------------------------------------------------------------
if [[ ! -s $CSV ]]; then
  echo "run_id,date,host,cpu_model,turbo,pg_cpus,variant,language,layer,rep,workers,pool_size,scenarios,duration_s,scenarios_per_s,errors,rows,scen_mean,scen_p50,scen_p95,scen_p99,q1_p50,q1_p99,q2_p50,q2_p99,q3_p50,q3_p99,q4_p50,q4_p99,cpu_s,cpu_ms_per_scenario,rss_peak_mib" > "$CSV"
fi

batch=$(date +%Y%m%dT%H%M%S)
for rep in $(seq "$REPS"); do
  # alternate the order, so a slow drift (temperature, disk) affects both variants equally
  if (( rep % 2 )); then order=(cornucopia kysely); else order=(kysely cornucopia); fi
  for variant in "${order[@]}"; do
    reset_db
    row=$(client "$variant" --rep "$rep" --workers "$WORKERS" --warmup "$WARMUP" --iterations "$ITERATIONS")
    echo "$batch-$variant-r$rep,$(date -Iseconds),$host,$cpu_model,$turbo,${PG_CPUS//,/ },$row" >> "$CSV"
    echo "rep $rep $variant: $(cut -d, -f9,13 <<< "$row" | tr , ' ') (scenarios/s, p50 ms)"
  done
done
echo "appended $((REPS * 2)) rows to $CSV"
