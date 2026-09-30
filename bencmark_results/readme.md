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
