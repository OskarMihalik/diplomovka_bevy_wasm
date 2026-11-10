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
- Browser: **Firefox** for the thesis comparison. Chrome (Playwright's Chromium) was added later
  as a side measurement through the `browser` column (section 9).
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

---

## 7. The bevy_diplomovka app benchmark

The same runner also measures the **whole app** (egui, picking, outlines, the backend download),
which today is done by hand: log in, open project "models benchmarks", open a model, press F9,
copy the CSV from the console into `bencmark_results/fps.csv` and `load.csv`.

**Decisions:** web and native; the new rows **replace** the old `fps.csv` and `load.csv` (same
columns as now, no `engine,…` prefix); the backend runs against the local docker-compose
Postgres (`db/data`); the app is measured **as it is**, with its panels on screen; models are
named by file name like in the rendering matrices, because database ids differ between PCs.

### 7.1 Bench mode in bevy_diplomovka (only with the `debug_tools` feature)

The UI is drawn by egui into a canvas, so Playwright can't click it. Instead the app gets a bench
mode, switched on by options (`?key=value` on the web, `--key value` natively, parsed like
`bevy_rendering_benchmark/src/config.rs`):

| option | |
|---|---|
| `bench_model` | model id; without it the app works exactly as now (F9 still works) |
| `email`, `password` | the benchmark account the app logs in with |
| `runs`, `duration`, `warmup` | replace the `BENCHMARK_RUNS` and `BENCHMARK_DURATION` constants; `runs=0` = load only |
| `resolution` | fixed window / canvas size in physical pixels, default 2088x1550, instead of `fit_canvas_to_parent` |
| `pc`, `report`, `out`, `exit` | as in the rendering benchmark |

With `bench_model`, the app goes through the same path as a user, driven by its own events
instead of clicks:

1. `GameState::Auth` → trigger `LoginEvent` with `email`/`password` (the login button does the same)
2. `LoggedUser` exists → `GET /model/{id}` gives the model's `project_id` → insert
   `ThisProjectIsSelected` on that `ProjectData` (as clicking the project) → the app loads the
   project's models
3. the `ModelData` with the id appears → insert `ThisModelIsSelected` (as clicking "Open") →
   the existing loading path downloads `/assets/model/{id}.glb` and logs the load row
4. model shown → warmup → the F9 benchmark starts by itself → `runs` fps rows
5. every row goes through `report::row` (web) or is appended to `--out` (native), then
   `report::done` / exit; a login error, an unknown model, a failed load or a panic go through
   `report::error`

Also: the app's native `date()` uses local time (as fixed in the rendering benchmark), and
`index.html` gets the same `window.benchReport` snippet.

### 7.2 Runner

- platforms `app-native` and `app-web`; `model` is the file name in the models folder, as for
  the other platforms (`barling_wrecks`)
- build: `cargo build -p bevy_diplomovka --release --features debug_tools` and
  `trunk build --release --cargo-profile wasm-release --features debug_tools --dist dist-bench`
  (a separate dist, so the deployed `dist` isn't replaced by a debug_tools build); served at
  `/app/`
- before the first app job, only through the backend's API (no SQL, no ids in the matrix):
  - `docker compose up -d postgres backend` if `http://localhost:4000` doesn't answer
  - log in as `bench@bench.local`, or `POST /register` it
  - project "benchmarks" of that account, `POST /project` if missing
  - every model of the matrix: uploaded from the models folder with
    `POST /model/{project}/{file name}` (multipart, limit 1 GiB, the airport is 0.96 GiB) unless a
    model of that name exists **and** its file downloads from `/assets/model/{id}.glb` (the
    backend container keeps uploads in its own filesystem, a recreated container loses them)
  - the id found this way is passed to the app as `bench_model`; `model_name` in the CSV is the
    file name, as in the rendering CSVs
- the page (runner origin `127.0.0.1:8090`) downloads from the backend (`localhost:4000`):
  cross-origin, allowed by the backend's `CorsLayer` (any origin), and compatible with COEP
  `require-corp` because it is a CORS request, so frame times stay at 20 µs
- rows go to `fps.csv` / `load.csv` (the file names the app reports); the expected-row check uses
  them for app jobs
- `matrices/app.csv`: the six models of `main.csv` on both platforms, London and the airport
  native only (wasm out of memory), 5 launches × one 60 s orbit like `main.csv`

### 7.3 Done (2026-10-16)

- bevy_diplomovka: `src/bench_mode.rs` (options, login → project → model → benchmark), `src/report.rs`,
  `pc` and local time in `system_info.rs`, the F9 benchmark takes `runs`/`duration` and reports its
  rows and `done`, `index.html` has `window.benchReport`. Without `bench_model` nothing changes.
- runner: `lib/app.js` prepares the backend through the API (tested: registers, creates the project,
  uploads, and reuses everything on the second run); `app-native` / `app-web` jobs; `matrices/app.csv`
  and two app rows in `smoke.csv`.
- the old `fps.csv` and `load.csv` are deleted (git history keeps them).
- tested: the app rows of `smoke.csv` passed natively and in Firefox: Intel GPU, 2088x1550, local
  dates, `model_name` = file name, sub-millisecond frame times on the web.
- `distance` in bench mode sets the radius of the app's PanOrbitCamera (it starts at (0, 1.5, 5)
  like the rendering benchmark's camera, so it is the same orbit); `fps.csv` and `load.csv` start
  with a `camera_distance` column (the camera's actual radius, also for a manual F9). `app.csv`
  uses the distances of `main.csv`. Tested at 20 m natively and on the web.

---

## 9. Chrome (2026-11-05)

`browser` column, `firefox` (default) or `chrome` = Playwright's Chromium build
(`npx playwright install chromium`, 153.0.8010.12 with Playwright 1.63.0; the user agent and the
`browser` column say Chrome, `browser_version` is the reduced `153.0.0.0`). Native Wayland, WebGL2
through ANGLE on OpenGL ES on the Intel GPU, cross-origin isolated (5 µs timer steps).

- `--kiosk` is ignored for the windows Playwright opens over CDP (978x1131), so the runner makes
  the window fullscreen with `Browser.setWindowBounds`: 2048x1280 CSS px like Firefox.
- Uncapping, `building` (light) and `barling_wrecks` (GPU-bound), legion, Intel:

  | flags | building bevy / babylon | p50 / p99 ms (bevy) | barling bevy / babylon |
  |---|---|---|---|
  | none | 137 / 87 fps | 6.09 / 12.7 (165 Hz) | 51 / 55 fps |
  | `--disable-gpu-vsync` | 138 / 107 fps | 6.08 / 12.6 (165 Hz) | 52 / 50 fps |
  | both + `--disable-frame-rate-limit` | **193 / 248 fps** | 2.12 / 76 | 54 / 57 fps |
  | X11 + `--disable-gpu-vsync` | 69 / 108 fps | 7.07 / 93 | |
  | Firefox, `layout.frame_rate=0` | 129 / 152 fps | 7.64 / 10.7 | 50 / 53 fps |

- Only both flags uncap Chrome. Then `requestAnimationFrame` runs ahead of the GPU: many
  near-zero frame times (babylon p50 0.20 ms on barling_wrecks) and stalls of hundreds of ms when
  the GPU queue is full. `avg_fps` (frames / time) is throughput and comparable with Firefox; the
  percentiles, `median_fps` and `low_1_fps` describe the submission of frames, not frames on
  screen, and aren't comparable.
- `bencmark_results/src/data.rs` ignores Chrome rows, so they don't mix with the Firefox
  platforms.

---

## 10. WebGPU (2026-11-05)

`api` column, `webgl2` (default) or `webgpu`, for `bevy-web` and `babylon-web`. Bevy: the
`webgpu` cargo feature (overrides `webgl2`), a second dist; wgpu only takes the adapter's
`description` for the name, which both browsers leave empty, so the webgpu build asks the browser
for the adapter itself and writes `vendor architecture` like Babylon. Babylon 9.29:
`WebGPUEngine` + `initAsync`; its WGSL shaders cover the scene, glslang/twgsl aren't loaded (the
cross-origin isolated page would block them). In Firefox one of Babylon's internal shaders
(`CopyVideoToTexture`) fails validation at startup; it's for video textures, rendering works.

- Firefox 155: WebGPU only with `dom.webgpu.enabled`; adapter info empty.
  `allow-present-without-readback`, 15 s orbit, Intel:

  | | readback (default) | without readback |
  |---|---|---|
  | Bevy barling_wrecks | 37.7 fps, p50 26.3 ms | **53.8 fps**, p50 18.2 ms |
  | Babylon barling_wrecks | 32.4 fps | 32.1 fps |
  | Bevy building | 67.7 fps | 73.7 fps |
  | Babylon building | 44.4 fps, p99 83.9 ms | 49.4 fps, p99 21.6 ms |

- Chrome 153: adapter `intel gen-12lp` (Vulkan) with `--enable-unsafe-webgpu
  --enable-features=Vulkan`; default: no adapter, only the first flag: SwiftShader. Uncapped like
  WebGL2, with the same requestAnimationFrame run-ahead (barling_wrecks babylon p50 0.39 ms,
  p99 416 ms), so only `avg_fps` is comparable.
- First smoke run (barling_wrecks, 10 s, Firefox still with readback): Bevy Firefox 37.9, Chrome
  76.2 fps; Babylon Firefox 32.4, Chrome 52.2 fps; native Bevy 44.6 fps.
- **Buffer limit (2026-11-06):** batch `20261105-221325` (webgpu.csv) gave Babylon 63 fps on
  london_city and 30 on national_airport_by_aditya, 8.5× and 6.6× its WebGL2. Their vertex
  buffers (353 MB, 650 MB) are over WebGPU's default `maxBufferSize` of 256 MiB; Babylon asked for
  the default limits, the buffer creation failed and those frames weren't drawn (in Chrome every
  command buffer was invalid). The runner didn't notice, a validation error isn't a page error.
  Fixed: `setMaximumLimits: true`, the adapter's limits like Bevy (wgpu's `adapter.limits()` with
  the default `Functionality` priority); london_city then 6.8 fps in Firefox, 7.3 in Chrome (WebGL2
  7.4). Babylon's page now reports any uncaptured WebGPU error as a job failure (except the
  `CopyVideoToTexture` one above): without the fix both london_city jobs fail with the buffer size
  error. The 20 jobs' rows were removed from rendering_fps.csv and rendering_load.csv (40 rows) and
  from the batch's done.txt; `--resume 20261105-221325` measures them again (~32 min). The other
  100 jobs had no WebGPU errors.
- The resume (2026-11-06, 20/20 passed, no WebGPU errors) ran without `--pc`, so its 40 rows got
  the host name `omih-83df` and the results tool skipped them; same laptop (meta.json: hostname,
  CPU, kernel, Mesa, power profile, commit), relabelled `legion`. `--resume` now takes the pc of
  the batch and refuses another `--pc`. london_city 6.8 fps (Firefox) / 7.6 (Chrome), WebGL2 7.4;
  national_airport_by_aditya 4.2 / 4.5, WebGL2 4.6.

