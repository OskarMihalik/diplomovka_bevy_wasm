trunk runned with these settings:

`trunk serve --release --cargo-profile wasm-release --features debug_tools`

firefox run with uncapped framerate in the settings about:config:

`layout.frame_rate: 0`

# Maximum model size

The error is an out-of-memory abort, not a bug in your code. The trace goes `Vec::from_iter` → `handle_alloc_error` → `rust_oom`, which is the WASM allocator failing to grow. A `wasm32` build can address at most 4 GB, and this model doesn't fit.

**Why london_city.glb is too big**

The file is 866 MB, with no textures. It holds 14.7M vertices and 28.5M indices, split into COLOR_0, NORMAL, POSITION, TEXCOORD_0 and TEXCOORD_1.

Loading it in Bevy 0.19 on the web makes four full copies of the file before any mesh is built:

| Step                                           | Where                              | Copy   |
| ---------------------------------------------- | ---------------------------------- | ------ |
| `fetch` → `Uint8Array::to_vec()`               | `bevy_asset/src/io/wasm.rs:108`    | 866 MB |
| `reader.read_to_end(&mut bytes)`               | `bevy_gltf/src/loader/mod.rs:1172` | 866 MB |
| `gltf::Gltf::from_slice` → `blob.into_owned()` | `gltf/src/lib.rs:301`              | 866 MB |
| `load_buffers` → `blob.into()`                 | `bevy_gltf/src/loader/mod.rs:1938` | 866 MB |

That's about 3.5 GB. The loader then starts turning attributes into `Vec<[f32; 3]>`, which needs close to another 1 GB, and it crashes at `convert_attribute`. As a rough rule, peak memory is about 4× the file size plus the decoded meshes, so anything much over 600–700 MB won't load. `national_airport_by_aditya.glb` (1.03 GB) will fail the same way. `administrative_and_warehouse_complex.glb` (353 MB) should load.

**Options**

1. **Shrink the model offline.** This is the practical fix. The model has no images, so both UV sets are dead weight. Removing them saves about 210 MB in the file and about 235 MB after decoding:
   ```sh
   npx @gltf-transform/cli prune london_city.glb london_city_pruned.glb
   npx @gltf-transform/cli weld london_city_pruned.glb london_city_small.glb
   ```
   `prune` drops attributes that no material texture uses. `weld` merges duplicate vertices. If it's still too big, add `simplify`. Don't use meshopt or Draco compression, because Bevy won't decode them.
2. **Guard against it in the app.** A WASM OOM can't be caught; it kills the whole tab. The app could check the model's size, either from the backend or the `Content-Length` header, before calling `asset_server.load` in [building.rs:265](bevy_diplomovka/src/building.rs#L265). If it's over a limit (say about 600 MB), show an egui-toast error instead.
3. **Cut the copies in Bevy.** This would mean forking or patching `bevy_gltf` so it borrows the blob instead of copying it, which removes up to two of the four copies. It's a lot of work for a thesis, and even at two copies this model would sit near the limit.

This could also be a finding for the thesis: WebAssembly's 4 GB memory limit combined with Bevy's loader copying the file makes about 600–700 MB the largest glb you can load in the browser, while a native build handles these files fine.

I can add the size guard from option 2 if you'd like.

# Native Bevy on the asus (Ivy Bridge HD Graphics 4000)

**Picking the Intel GPU**

Mesa's Vulkan driver for Ivy Bridge (`hasvk`) is incomplete and reports conformance version 0.0.0. wgpu hides non-conformant Vulkan adapters, so native Bevy falls back to `llvmpipe` (CPU rendering, ~1.6 fps). Run the benchmark with the override:

`WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER=1 node rendering_runner/run.mjs rendering_runner/matrices/main.csv --pc asus`

With it, Bevy runs on `Intel(R) HD Graphics 4000 (IVB GT2)` over Vulkan.

**Large models run out of GPU memory**

In run `logs/rendering/20261022-133348`, native Bevy failed all 5 repeats of `cai_mep_port_vietnam`, `london_city` and `national_airport_by_aditya`. The first error is `Out of Memory` (cai_mep, in `mesh slab resize encoder`, right after the first frame) or `DeviceLost: Out of memory` (london_city). Babylon and Bevy on wasm load the same models on the same GPU without problems.

The cause is the driver, not the engine:

- `hasvk` advertises a single 1.5 GiB heap, sized from the Ivy Bridge 2 GiB graphics aperture. It counts every allocation against that heap and refuses one that would go over it, even though system RAM is still free. No setting raises it.
- The web builds go through WebGL2 → Firefox → OpenGL, on Mesa's `crocus` driver. OpenGL has no such budget: buffers live in system RAM and the kernel only has to fit the objects a single draw batch uses into the aperture, swapping the rest in and out.

Native Bevy also needs more GPU memory for the same model:

- GPU preprocessing and GPU clustering are on (they need compute shaders, which WebGL2 doesn't have), adding buffers like `indexed preprocess work item buffer` and `bin unpacking metadata buffer`.
- Bevy packs meshes into shared slab buffers. Growing a slab allocates a bigger one and copies into it, so both exist for a moment. cai_mep failed during exactly this step.
- Babylon gives each mesh its own buffer and never makes that copy.

The share of each of these hasn't been measured; the 1.5 GiB Vulkan heap alone explains the failures.

Finding for the thesis: on the asus, native Bevy (Vulkan) cannot render these three models because the `hasvk` driver limits the GPU heap to 1.5 GiB, while the web versions run through OpenGL, which has no such limit. This is a limit of the old GPU's Vulkan driver, not of Bevy. Running native Bevy through OpenGL (wgpu's `gles` feature) would remove the limit and put it on the same driver as the web builds.

# parameters

Intel® UHD Graphics for 14th Gen Intel® Processors
Operating System: Kubuntu 26.04 LTS
KDE Plasma Version: 6.6.6
KDE Frameworks Version: 6.24.0
Qt Version: 6.10.2
Kernel Version: 7.0.0-34-generic (64-bit)
Graphics Platform: Wayland
Processors: 28 × Intel® Core™ i7-14700HX
Memory: 32 GiB of RAM (31,1 GiB usable)
Graphics Processor 1: Intel® Graphics
Graphics Processor 2: NVIDIA GeForce RTX 4070 Laptop GPU
Manufacturer: LENOVO
Product Name: 83DF
System Version: Legion Pro 5 16IRX9
