# Rendering benchmark runner plan

Runs the rendering and loading benchmarks (`bevy_rendering_benchmark` native and wasm,
`babylon_rendering_benchmark`) from a matrix CSV, without opening pages or copying CSV rows from
the browser console by hand. Everything new goes into `rendering_runner/`. The benchmarks change
only in the small ways listed in section 3.

---

## 1. Goals and decisions

| Goal | How |
|---|---|
| One command runs a whole set of benchmarks | `node rendering_runner/run.mjs <matrix.csv>` |
| The same matrix covers native and browser | `platform` column: `bevy-native`, `bevy-web`, `babylon-web` |
| Results go into the existing CSVs | rows are appended to `bencmark_results/rendering_fps.csv` and `rendering_load.csv`, with no new columns |
| Loading can be measured separately with many repetitions | `runs=0` = load only; every repetition is a new process or browser, so each load starts cold |
| Long overnight batches survive failures and interruptions | failed jobs are logged and skipped, `--resume <batch>` |

**Decisions already made:**
- Browser: **Firefox only**, no Chrome.
- The browser is driven with **Playwright** (`playwright` pinned to an exact version in
  `rendering_runner/package.json`).
- Firefox is **Playwright's own Firefox build** (`npx playwright install firefox`), not the snap.
  Playwright can't drive a normal Firefox install; its build is Firefox stable with a small
  automation patch (Juggler). Pinning `playwright` pins the Firefox version, which is recorded in
  `meta.json` (`browser.version()`). It also avoids the snap's own Mesa, a different driver than
  native Bevy uses. The thesis should say the measurements ran in Playwright's Firefox build.
- The runner also does **native** jobs and replaces `bevy_rendering_benchmark/run_native.sh`.
- The runner is written in **Node**, like the Babylon benchmark.
- The CSV columns don't change, so `bencmark_results/src/data.rs` keeps working.
- **Everything is re-measured with the runner**, web and native, in the same batches. The old
  manual rows aren't mixed with runner rows (step 0: other Firefox version, other machine state).
- **Sub-millisecond frame times** in the browser: Firefox normally rounds `performance.now()` to
  1 ms. The server sends COOP/COEP headers, so the pages are cross-origin isolated and Firefox
  rounds to 20 µs instead (tested in step 1: `crossOriginIsolated` true, 0.02 ms steps, frame
  times like 17.64 ms). No privacy pref is changed.

---

## 2. The matrix CSV

Each column is a benchmark option with the same name as today (`model`, `runs`, `duration`,
`warmup`, `gpu`, `resolution`, `msaa`, `shadows`, `distance`, `id`, `name`, `version`), plus the
runner's own columns:

| column | meaning |
|---|---|
| `platform` | `bevy-native`, `bevy-web` or `babylon-web` |
| `model` | a file name in the models folder without `.glb` (`barling_wrecks` → `~/Documents/diplomovka_models/barling_wrecks.glb`) or a path; the CSV's `model_name` comes from the file name as in the existing rows |
| `repeat` | how many times to start a fresh process or browser; each gives one load row (default 1) |
| `env` | extra environment variables separated by `;`, e.g. `WGPU_BACKEND=gl` or `__NV_PRIME_RENDER_OFFLOAD=1;__GLX_VENDOR_LIBRARY_NAME=nvidia` |
| `expect_gpu` | optional: text the reported GPU name must contain, otherwise the job fails (catches runs on the wrong GPU) |

Rules:
- An empty cell means the benchmark's own default.
- A cell with space-separated values expands into every combination: `msaa=1 4` with
  `distance=5.22 20 60` gives 6 jobs.
- `runs` stays the number of measured orbits inside one start. `runs=0` means load only.
- Any other column is passed through as an option, so new benchmark options need no runner change.

Example:

```csv
platform,model,repeat,runs,duration,warmup,gpu,msaa,distance,env
bevy-native,barling_wrecks cai_mep_port_vietnam,1,5,60,5,low,4,5.22,
bevy-native,barling_wrecks,1,5,60,5,low,4,5.22,WGPU_BACKEND=gl
bevy-web,barling_wrecks cai_mep_port_vietnam,1,5,60,5,low,4,5.22,
babylon-web,barling_wrecks cai_mep_port_vietnam,1,5,60,5,low,4,5.22,
bevy-web,barling_wrecks,10,0,,,low,,,
babylon-web,barling_wrecks,10,0,,,low,,,
bevy-native,administrative_and_warehouse_complex,1,3,60,5,low,1 4,5.22 20 60,
```

**Job order:** jobs run in rounds: repetition 1 of every row, then repetition 2 of every row, and
so on. Temperature drift then affects all platforms equally (the same reason query_bench
alternates its order). Between jobs the runner waits `--cooldown` seconds (default 10).

---

## 3. Changes to the benchmarks

The same changes in Bevy (native and wasm) and Babylon:

1. **`report` URL parameter** (web only). With `?report=/report`, every CSV row is POSTed there as
   `{file, header, row}`. When the benchmark finishes it POSTs `{done: true}`, on failure `{error}`.
   - Babylon: in `emitCsv` (`src/csv.js`), at the end in `src/bench.js`, and in the load-failure
     path in `src/main.js`.
   - Bevy: a `window.benchReport` function in `index.html`, called through a `wasm_bindgen` extern
     from `emit_csv` and the finish path in `src/bench.rs`, the load-failure path in
     `src/load_timing.rs`, and a panic hook.
   - POST rather than parsing the console or polling with `page.evaluate`: the results arrive as
     structured data, and the runner adds nothing to the page's main thread while frame times are
     measured. The console is still saved to the job's log through `page.on("console")`.
2. **`pc` option** (`?pc=` / `--pc`). Overrides `PC_NAME`, which is currently fixed at build time,
   so the runner can set it.
3. **`runs=0` = load only.** Today `runs` is forced to at least 1. With `runs=0` the benchmark
   reports done (web) or exits (native) right after the load row.

Both READMEs get a section on these options and a pointer to the runner.

---

## 4. The runner

```sh
node rendering_runner/run.mjs <matrix.csv> --pc legion
    [--out bencmark_results] [--cooldown 10] [--no-build] [--dry-run]
    [--resume <batch>] [--port 8090] [--models ~/Documents/diplomovka_models]
```

### 4.1 Build
Only the platforms the matrix uses, skipped with `--no-build`:
- `cargo build -p bevy_rendering_benchmark --release`
- `trunk build --release --cargo-profile wasm-release` in `bevy_rendering_benchmark/` (`build`,
  not `serve`, so no dev server or file watcher runs during measurements)
- `npm run build` in `babylon_rendering_benchmark/`

### 4.2 Native jobs
Start `target/release/bevy_rendering_benchmark --model <path> --out <out> --pc <pc> --exit on`
plus the row's options and environment variables. stdout and stderr go to the job's log. A
non-zero exit code or a timeout (load timeout + warmup + runs × duration + margin) is a failure.

### 4.3 Web jobs
- **One HTTP server, one origin** (no CORS):
  - `/bevy/` → `bevy_rendering_benchmark/dist`
  - `/babylon/` → `babylon_rendering_benchmark/dist`
  - `/models/` → streams the models folder (`--models`, default `~/Documents/diplomovka_models`),
    so `download_parse_ms` includes the download as in the app
  - `POST /report` → results of the running job
- **Firefox** through Playwright:
  - `firefox.launch({ headless: false, args: ["--kiosk"], firefoxUserPrefs, env })`, a new
    browser per job: a fresh process with the row's `env`, and a new temporary profile, so the
    HTTP cache and shader cache are empty
  - `newContext({ viewport: null })`: no viewport or device-scale emulation, the page sizes its
    canvas itself as in manual runs; kiosk because headless can fall back to software rendering,
    and a 2088×1550 canvas only fits the 2560×1600 screen without toolbars
  - `firefoxUserPrefs`: `layout.frame_rate=0`; nothing else was needed (step 0)
  - GPU selection through the `env` column (PRIME offload variables); `gpu` only sets the WebGL
    power preference
- **One job:** open `/<engine>/?model=/models/<name>.glb&report=/report&pc=…&<options>`, wait for
  `done` or `error`, close the browser. Launch, `newPage` and the first navigation each get their
  own short timeout: they hang while Firefox can't draw (locked screen, section 4.5). Page errors, the browser closing by itself (e.g. wasm out
  of memory on model 19) and the timeout are failures; the batch continues.

### 4.4 Results and logs
- Rows are appended to `<out>/rendering_fps.csv` and `<out>/rendering_load.csv`. The header is
  written only for a new file; if an existing file has a different header, the runner stops.
- Per batch, in `bencmark_results/logs/rendering/<batch>/`:
  - `<job>.log`: the job's console or stdout
  - `matrix.csv`: a copy of the matrix
  - `meta.json`: git commit, Playwright and Firefox versions, kernel, Mesa version, AC power,
    and the processes using the most CPU when the batch starts
  - `done.txt`: finished jobs, for `--resume`
- Each job checks that it got the expected rows (1 load row + `runs` fps rows) and, if set, that
  the GPU name matches `expect_gpu`.

### 4.5 Long runs
- `--dry-run` lists all jobs with an estimate of the total time.
- `--resume <batch>` skips the jobs in that batch's `done.txt`.
- Before starting, a warning if the laptop isn't on AC power, and a list of running apps that
  share the GPU (Discord, Obsidian, VS Code, another Firefox) to close first.
- **The screen must stay unlocked and the benchmark window on top.** On Wayland, Firefox draws
  nothing while the screen is locked: `newPage` hung for 238 s until the screen was unlocked, and
  a running benchmark stops. The runner:
  - holds `kde-inhibit --power --screenSaver` for the whole batch and calls
    `org.freedesktop.ScreenSaver.SimulateUserActivity` every 30 s (tested: works on this Plasma)
  - checks `loginctl show-session -p LockedHint` before and after each job; a job that saw a
    locked screen fails and is repeated after the unlock
  - the laptop isn't used during a batch: any window on top of the kiosk window, even VS Code
    redrawing, changes the frame times
- At the end, a summary of passed and failed jobs; non-zero exit code if any failed.

---

## 5. Steps

0. **Trial: done**, results in section 6. Still open: the PRIME variables for the RTX 4070.
1. **Done:** benchmark changes from section 3, both READMEs. Tested: native `--runs 0 --pc`,
   web rows + `done`, load only, bad settings and a missing model reported as `error`, in both
   engines. Not tested: the panic hook (no easy way to trigger a panic).
2.–5. **Done:** `run.mjs` with `lib/matrix.js`, `lib/server.js`, `lib/jobs.js`, `lib/host.js`;
   usage in [README.md](README.md). `run_native.sh` removed. Differences from the plan above:
   - native jobs write to a folder per job (`--out`), the runner reads the rows back, so native
     and web rows are checked the same way before anything is appended
   - before the first web job, a check page records the Firefox version, the WebGL2 renderer,
     cross-origin isolation and the timer step in `meta.json`
   - `#` comment lines in matrices; `platform` can also take several values
6. **Done:** `matrices/smoke.csv`, `main.csv`, `load.csv`. No `scaling.csv`: the scalability
   figure plots fps against the triangles of the six models, not a distance sweep.
   `main.csv` uses 5 launches × one 60 s orbit instead of one launch × 5 orbits: independent
   samples, and a cold load row for every launch.
7. **Done:** `smoke.csv` passed on all 3 platforms into a scratch `--out`, and
   `cargo run -p bencmark_results` reads the rows. Also tested: `expect_gpu` mismatch fails the
   job without appending rows, `--resume` skips finished jobs.

**Next:** close the other apps and run `main.csv` and `load.csv` into `bencmark_results`.
Still open: the RTX 4070 through PRIME variables (`env` column), if a dedicated-GPU reference is
wanted.

---

## 6. Step 0 results (2026-10-16)

Script: `rendering_runner/trial.mjs` (throwaway). Playwright 1.63.0, its Firefox **155.0**; the
snap Firefox is **157.0**.

- Both benchmarks run in Playwright's Firefox on the Intel GPU (`Intel(R) HD Graphics, or
  similar`, the same string as the manual rows), canvas 2088×1550, kiosk window 2048×1280 CSS px
  at 1.25 = the full 2560×1600 screen.
- `layout.frame_rate=0` works: Hong Kong at distance 10 reaches 66 fps.
- Firefox runs as a native Wayland client, not through XWayland.
- Frame times are whole milliseconds (`frame_min_ms` 8.00, 12.00), as in the manual rows: Firefox
  rounds `performance.now()` to 1 ms. `privacy.reduceTimerPrecision=false` would give finer
  percentiles, but those rows couldn't be compared with the old ones.
- The snap Firefox can also run without Playwright (fresh profile, console on stdout through
  `devtools.console.stdout.content`). AppArmor only lets processes inside the snap stop it:
  `snap run --shell firefox -c "pkill -f <profile>"`.

**Three-way comparison**, Hong Kong, distance 10, MSAA 4, one 60 s orbit per launch, average fps
(interrupted after 9 of 12 launches):

| engine | Playwright → FF 155 | FF 155 without Playwright | snap FF 157 | manual FF 157 (old rows) |
|---|---|---|---|---|
| Bevy | 66.4, 65.9 | 65.7, 66.3 | 60.4, 60.6 | 88.2 |
| Babylon | 63.1 | 63.4 | 72.2 | 91.8 |

- **Playwright itself costs nothing**: the same Firefox with and without it is within 1%.
- **The Firefox version matters, differently per engine**: in 157, Bevy is 9% slower and Babylon
  14% faster than in 155. So all browser rows compared in the thesis must come from one Firefox
  build, and the old manual rows (157) can't be mixed with new runner rows (155).
- **All three are ~25–30% below the old manual rows**, including the same snap 157. That is the
  machine state today (VS Code, Discord, Obsidian open; possibly GPU clocks or power profile), not
  the runner. So native should also be re-measured in the same batches, and the background apps
  closed (section 4.5).
