// Rendering benchmark: opens one glb model the way bevy_rendering_benchmark does and measures
// how long it takes to load and how fast it renders while the camera orbits it. See README.md.

import { BaseTexture, Engine, LoadAssetContainerAsync, Scene, WebGPUEngine } from "@babylonjs/core";
import "@babylonjs/loaders/glTF";

import { Bench } from "./bench.js";
import { configFromUrl } from "./config.js";
import { LoadTiming } from "./loadTiming.js";
import { report } from "./report.js";
import { setupScene, spawnModel } from "./scene.js";

const canvas = document.getElementById("babylon");
const picker = document.getElementById("picker");
const pickerText = document.getElementById("picker-text");

let config;
try {
  config = configFromUrl();
} catch (error) {
  console.error(error.message);
  report({ error: error.message });
  throw error;
}

// physical pixels, so every machine and browser renders the same amount: the backbuffer is the
// canvas' css size, not scaled by devicePixelRatio
canvas.style.width = `${config.width}px`;
canvas.style.height = `${config.height}px`;
// no antialiasing on the canvas, MSAA happens offscreen (scene.js)
const engine = config.webgpu ? await createWebGpuEngine() : new Engine(
  canvas,
  false,
  { powerPreference: config.powerPreference, stencil: true, preserveDrawingBuffer: false },
  false,
);
// no .manifest requests next to the model
engine.enableOfflineSupport = false;
const scene = new Scene(engine);
const { camera, shadowGenerator } = setupScene(scene, config);
const bench = new Bench(engine, scene, camera, config);

let frame = 0;
let model = null;
let loading = null;
let lastFrame = performance.now();
// requestAnimationFrame, uncapped only if the browser is (see README.md)
engine.runRenderLoop(() => {
  const now = performance.now();
  const delta = (now - lastFrame) / 1000;
  lastFrame = now;
  if (loading?.finish(frame, model, engine, config)) {
    loading = null;
  }
  bench.update(delta, model !== null && loading === null);
  scene.render();
  frame += 1;
});

/**
 * Babylon's WebGPU engine with its own WGSL shaders; a material without them would load the
 * GLSL compilers from Babylon's CDN, which the cross-origin isolated page blocks
 */
// Firefox's WebGPU rejects the shader of Babylon's internal video-to-texture copy, which it
// creates at startup; no video is used here. The first error has no label. A model's own broken
// shader would still fail the job through the invalid pipeline using it, whose label isn't a
// video one.
const HARMLESS_WEBGPU_ERRORS = [
  /CopyVideoToTexture|InternalVideoPipeline/,
  /^Shader module creation failed: Shader validation error$/,
];

async function createWebGpuEngine() {
  try {
    const engine = new WebGPUEngine(canvas, {
      antialias: false,
      powerPreference: config.powerPreference,
      stencil: true,
      // the adapter's limits like Bevy (wgpu's adapter.limits()): with WebGPU's defaults the
      // 256 MiB maxBufferSize rejects the vertex buffers of london_city (353 MB) and
      // national_airport_by_aditya (650 MB), and nothing is drawn
      setMaximumLimits: true,
    });
    await engine.initAsync();
    // a validation error means frames that weren't drawn as measured: the job fails
    engine._device.addEventListener("uncapturederror", (event) => {
      const message = event.error?.message ?? String(event.error);
      if (HARMLESS_WEBGPU_ERRORS.some((pattern) => pattern.test(message))) return;
      console.error(`WebGPU error: ${message}`);
      report({ error: `WebGPU error: ${message}` });
    });
    return engine;
  } catch (error) {
    const message = `WebGPU init failed: ${error.message ?? error}`;
    console.error(message);
    report({ error: message });
    throw error;
  }
}

/** `source` is a url or the bytes of a picked file */
async function loadModel(source, fileName) {
  loading = new LoadTiming(frame);
  console.log(`model load: request started, ${fileName ?? source}`);
  try {
    const container = await LoadAssetContainerAsync(source, scene, {
      // bytes have no file name to tell glb from gltf
      pluginExtension: fileName?.toLowerCase().endsWith(".gltf") ? ".gltf" : ".glb",
    });
    // like Bevy's is_loaded_with_dependencies, every texture is decoded
    await new Promise((resolve) => BaseTexture.WhenAllReady(container.textures, resolve));
    loading.parsed();
    model = spawnModel(scene, container, shadowGenerator);
    loading.spawned(frame);
  } catch (error) {
    console.error("model load failed:", error);
    report({ error: `model load failed: ${error.message ?? error}` });
    pickerText.textContent = `Model load failed: ${error.message ?? error}`;
    picker.classList.add("open");
    loading = null;
  }
}

if (config.model) {
  loadModel(config.model);
} else {
  showPicker();
}

/** Reads the picked model into memory, reading the file isn't part of the load time */
function showPicker() {
  picker.classList.add("open");
  let picked = false;
  const pick = async (file) => {
    if (!file || picked) return;
    picked = true;
    pickerText.textContent = `Reading ${file.name} (${(file.size / 1048576).toFixed(1)} MiB)…`;
    let bytes;
    try {
      bytes = new Uint8Array(await file.arrayBuffer());
      // id and name for the CSV come from the file name now, unless they are in the url
      Object.assign(config, configFromUrl(file.name));
    } catch (error) {
      picked = false;
      pickerText.textContent = `Can't read ${file.name}: ${error.message ?? error}`;
      return;
    }
    picker.classList.remove("open");
    console.log(`model picked: ${file.name}, ${(file.size / 1048576).toFixed(1)} MiB`);
    loadModel(bytes, file.name);
  };
  picker.querySelector("input").addEventListener("change", (e) => pick(e.target.files[0]));
  addEventListener("dragover", (e) => {
    e.preventDefault();
    picker.classList.add("drag");
  });
  addEventListener("dragleave", () => picker.classList.remove("drag"));
  addEventListener("drop", (e) => {
    e.preventDefault();
    picker.classList.remove("drag");
    pick(e.dataTransfer.files[0]);
  });
}
