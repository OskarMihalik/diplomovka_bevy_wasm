# rendering_runner

Runs the rendering and loading benchmarks ([bevy_rendering_benchmark](../bevy_rendering_benchmark)
native and wasm, [babylon_rendering_benchmark](../babylon_rendering_benchmark)) and the whole
[bevy_diplomovka](../bevy_diplomovka) app from a matrix CSV, and appends the rows to
`bencmark_results/rendering_fps.csv` and `rendering_load.csv` (the app: `fps.csv` and
`load.csv`). The web benchmarks run in Playwright's Firefox. Why it works the way it does:
[PLAN.md](PLAN.md).

## Setup

```sh
cd rendering_runner
npm install
npx playwright install firefox
```

Also needed: `cargo`, `trunk`, `npm install` in `babylon_rendering_benchmark`, and for the app
`docker compose` (the runner starts Postgres and the backend if they aren't running).

## Run

```sh
node rendering_runner/run.mjs rendering_runner/matrices/smoke.csv --pc legion --dry-run
node rendering_runner/run.mjs rendering_runner/matrices/smoke.csv --pc legion
```

| option | default | |
|---|---|---|
| `--pc NAME` | host name | `pc` column |
| `--out DIR` | `bencmark_results` | where the CSVs are appended, logs go to `DIR/logs/rendering/<batch>/` |
| `--models DIR` | `~/Documents/diplomovka_models` | folder of the models named in the matrix |
| `--cooldown S` | 10 | pause between jobs |
| `--no-build` | | skip `cargo build`, `trunk build`, `npm run build` |
| `--dry-run` | | list the jobs and the estimated time, run nothing |
| `--resume BATCH` | | continue a stopped batch, skips the jobs in its `done.txt` |
| `--port N` | 8090 | port of the local server for the web benchmarks |

Ctrl+C stops after killing the current job; `--resume` continues later.

**Before a batch:** plug in the charger, close Discord, Obsidian, VS Code, other browsers (the
runner lists what is open), and don't use the laptop until it finishes. The runner keeps the
screen from locking (`kde-inhibit` and simulated activity every 30 s), because a locked screen
stops Firefox from drawing; a job that saw a locked screen is repeated after the unlock.

## Matrix

One row per configuration. A cell with several values separated by spaces expands into every
combination, an empty cell means the benchmark's default, `#` lines are comments.

```csv
platform,model,distance,repeat,runs,duration,warmup,gpu,resolution,msaa,shadows,expect_gpu
bevy-native bevy-web babylon-web,barling_wrecks cai_mep_port_vietnam,5.22,5,1,60,5,low,2088x1550,4,on,Intel
bevy-native,administrative_and_warehouse_complex,19000,1,1,60,5,low,2088x1550,1 4,on,Intel
```

| column | |
|---|---|
| `platform` | `bevy-native`, `bevy-web`, `babylon-web`, `app-native`, `app-web` |
| `model` | name in the models folder (`barling_wrecks`), or a path |
| `repeat` | launches of a new process or browser (a cold load each), default 1 |
| `runs` | measured orbits per launch, default 5; 0 = load only |
| `env` | environment variables, `A=1;B=2`, e.g. `WGPU_BACKEND=gl` |
| `expect_gpu` | text the GPU name must contain, otherwise the job fails |
| anything else | passed to the benchmark: `duration`, `warmup`, `gpu`, `resolution`, `msaa`, `shadows`, `distance`, `id`, `name`, `version` |

Jobs run in rounds: launch 1 of every configuration, then launch 2, and so on, so slow drift
(temperature) affects every platform alike.

Matrices in `matrices/`:

- `smoke.csv`: every platform once, a few seconds each (~4 min)
- `main.csv`: the thesis comparison, 7 models × 3 platforms, 5 launches × 60 s (~2 h 30 min)
- `load.csv`: load only, 10 launches per model and platform (~1 h 35 min)
- `previous_fps.csv`: the configurations of the old manual `rendering_fps.csv`, 5 orbits per launch (~1 h 30 min)
- `app.csv`: the whole app, 7 models, native and web (~1 h 35 min)
- `nvidia.csv`: `main.csv` on the dedicated RTX 4070 as a reference (~2 h 30 min); the results
  tool writes it to `tables/fps_rtx.tex` and `ttest_rtx.tex`

## The NVIDIA GPU (`nvidia.csv`)

Natively `gpu=high` picks the RTX 4070. Firefox renders on it only with PRIME render offload,
`env` = `__NV_PRIME_RENDER_OFFLOAD=1;__GLX_VENDOR_LIBRARY_NAME=nvidia` (checked with the WebGL
renderer and `nvidia-smi`; under XWayland Firefox has no WebGL2). `expect_gpu=NVIDIA` fails any
job that falls back to the integrated GPU: native rows say `NVIDIA GeForce RTX 4070 Laptop GPU`,
Firefox `NVIDIA GeForce GTX 980, or similar` (its sanitized name). The laptop panel is driven by
the integrated GPU, so every frame is copied to it (hybrid mode). Disconnect monitors on the
NVIDIA GPU (HDMI) before the batch, the runner warns about them.

## The app (`app-native`, `app-web`)

The app is built with `debug_tools` (the web build into `bevy_diplomovka/dist-bench`) and started
in bench mode ([bench_mode.rs](../bevy_diplomovka/src/bench_mode.rs)): it logs in, selects the
project, opens the model and runs the F9 benchmark by itself, with its panels on screen. Before
the first app job the runner prepares the backend through its API: the account
`bench@bench.local`, its project "benchmarks", and every model of the matrix uploaded from the
models folder unless it is there and downloads. So the matrix names models by file name on every
PC; the ids found on this PC are in `meta.json`. Natively, `gpu=low` becomes `WGPU_POWER_PREF`.
`distance` sets the radius of the app's orbit camera (the same orbit as the rendering benchmark,
written to the `camera_distance` column); `msaa` and `shadows` don't apply, the app renders with
its own settings.

## Output

- Rows are appended to `<out>/rendering_fps.csv` and `rendering_load.csv` only when a job passed
  (it got 1 load row and `runs` fps rows, and the GPU matched `expect_gpu`).
- `<out>/logs/rendering/<batch>/`: one log per job (console or stdout), `matrix.csv`, `done.txt`,
  `failed.txt`, `build.log`, and `meta.json` (git commit, Firefox and Playwright versions, timer
  resolution, Mesa, kernel, power profile, open apps, results).
- Web pages are cross-origin isolated, so Firefox measures frame times in 20 µs steps instead of
  1 ms.
