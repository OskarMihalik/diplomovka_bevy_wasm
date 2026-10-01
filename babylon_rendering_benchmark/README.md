# babylon_rendering_benchmark

The same benchmark as [bevy_rendering_benchmark](../bevy_rendering_benchmark), in Babylon.js
(WebGL2). It opens one glb model the way `bevy_diplomovka` shows it (one shadow-casting point
light, model scaled to 0.25, camera orbiting from (0, 1.5, 5)) and measures:

- **load time**: download + parsing, mesh creation, first frame
- **frame times**: after a warmup that isn't measured, `runs` × `duration` s, one camera orbit
  per run

The CSV rows have the same columns as the Bevy benchmark's `rendering_load.csv` and
`rendering_fps.csv`, with `engine` = `babylon`, so they can be pasted into the same files.

## Run

```sh
cd babylon_rendering_benchmark
npm install
npm run build && npm run preview     # or `npm run dev` while changing the code
```

Open `http://localhost:8083/` and choose a .glb, or drop it on the page. It is read into memory,
so `download_parse_ms` is only parsing. The id and name in the CSV come from the file name
(`14.glb` → 14), or `?id=…&name=…`. Other options go in the url, e.g. `?gpu=low&msaa=1`.

To measure the download as well, serve the models with CORS and pass the url:

```sh
npx http-server backend/assets/models --cors -p 8090
```

`http://localhost:8083/?model=http://localhost:8090/14.glb`. The results appear in the browser
console. Uncap the browser frame rate first: in Firefox, set `layout.frame_rate` to `0` in
about:config. In Chrome, start it with `--disable-frame-rate-limit --disable-gpu-vsync`.

`PC_NAME="desktop" npm run build` sets the `pc` column.

## Options

`?key=value&...`:

| option       | default       |                                                              |
| ------------ | ------------- | ------------------------------------------------------------ |
| `model`      | picked        | url of a .glb, relative to the page; leave out to pick the file |
| `id`         | file name     | model id in the CSV, `14.glb` → 14                           |
| `name`       | known ids     | model name in the CSV, ids 12–18 are known                   |
| `version`    | 1             |                                                              |
| `runs`       | 5             |                                                              |
| `duration`   | 60            | seconds per run = one orbit                                  |
| `warmup`     | 5             | seconds, not measured                                        |
| `gpu`        | high          | `low` / `high` WebGL power preference                        |
| `resolution` | 2088x1550     | physical pixels                                              |
| `msaa`       | 4             | 1, 2, 4, 8                                                   |
| `shadows`    | on            | point light shadows                                          |
| `distance`   | 5.22          | camera distance from the center in m (model is scaled 0.25)  |

## How it matches the Bevy version

Babylon's defaults are replaced with Bevy's where they change what gets drawn or how much it
costs (see `src/scene.js`):

- right-handed coordinates, so the glTF needs no flip and the camera path is identical
- Bevy's default `PointLight`: range 20 m, 2048 px shadow cube map, intensity converted through
  Bevy's default exposure; Bevy's default ambient light as a uniform hemispheric light
- Bevy's default camera: 45° vertical fov, near 0.1, infinite reverse-Z depth (no far plane), tonemapping (ACES, Babylon has no
  TonyMcMapface)
- MSAA in an offscreen texture that is copied to the canvas, like Bevy's main texture, with
  exactly `msaa` samples; the canvas itself has no antialiasing
- no pointer picking

Differences that remain:

- shadow filtering: Poisson sampling, Babylon has no PCF for point lights
- Babylon draws glTF nodes that reuse a mesh as hardware instances (the loader's default)
- `texture_mib` is an estimate: RGBA8 plus mipmaps, which Babylon generates and Bevy doesn't
- `entities` counts Babylon scene nodes (meshes, transform nodes, lights, cameras)
- `first_frame_ms` is, like in Bevy, the first frame after the model was added; shaders
  compiling in the background may still hide parts of it, the warmup covers that
