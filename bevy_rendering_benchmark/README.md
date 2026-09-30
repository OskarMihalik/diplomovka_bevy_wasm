# bevy_rendering_benchmark

Opens one glb model exactly as `bevy_diplomovka` shows it (one shadow-casting point light, model
scaled to 0.25, camera orbiting from (0, 1.5, 5)), but without egui, the backend API, picking and
outlines. Measures:

- **load time**: download + parsing, mesh creation, first frame (like `bencmark_results/load.csv`)
- **frame times**: after a warmup that isn't measured, `runs` × `duration` s, one camera orbit
  per run (like `bencmark_results/fps.csv`, same logic as `debug_tools.rs` F9)

The CSV rows have the same columns as `load.csv` and `fps.csv`, with `engine,msaa,shadows,camera_distance` in front.

## Native

```sh
cargo run -p bevy_rendering_benchmark --release -- \
    --model backend/assets/models/14.glb --gpu low --out bencmark_results
# all models, one process each
bevy_rendering_benchmark/run_native.sh --gpu low
```

`--gpu low` picks the integrated GPU (the one the browser uses), `--gpu high` picks the dedicated one.
Set `WGPU_BACKEND=gl` to use OpenGL like the browser does instead of Vulkan.

## Web

Serve the models with CORS, then the benchmark:

```sh
npx http-server backend/assets/models --cors -p 8090
cd bevy_rendering_benchmark && trunk serve --release --cargo-profile wasm-release
```

Open `http://localhost:8082/?model=http://localhost:8090/14.glb`. The results appear in the
browser console. Uncap the browser frame rate first: in Firefox, set `layout.frame_rate` to `0` in about:config.
In Chrome, start it with `--disable-frame-rate-limit --disable-gpu-vsync`.

## Options

Native `--key value`, web `?key=value&...`:

| option       | default       |                                                            |
| ------------ | ------------- | ---------------------------------------------------------- |
| `model`      | required      | path to a .glb, or http(s) url (relative to the page on web) |
| `id`         | file name     | model id in the CSV, `14.glb` → 14                         |
| `name`       | known ids     | model name in the CSV, ids 12–18 are known                 |
| `version`    | 1             |                                                            |
| `runs`       | 5             |                                                            |
| `duration`   | 60            | seconds per run = one orbit                                |
| `warmup`     | 5             | seconds, not measured                                      |
| `gpu`        | high          | `low` / `high` wgpu power preference                       |
| `resolution` | 1920x1080     | physical pixels                                            |
| `msaa`       | 4             | 1, 2, 4, 8                                                 |
| `shadows`    | on            | point light shadows                                        |
| `distance`   | 5.22          | camera distance from the center in m (model is scaled 0.25) |
| `out`        | –             | native: directory to append the CSVs to                    |
| `exit`       | on            | native: close the window when done                         |
