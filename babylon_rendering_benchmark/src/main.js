// Rendering benchmark: opens one glb model the way bevy_rendering_benchmark does and measures
// how long it takes to load and how fast it renders while the camera orbits it. See README.md.

import { BaseTexture, Engine, LoadAssetContainerAsync, Scene } from "@babylonjs/core";
import "@babylonjs/loaders/glTF";

import { Bench } from "./bench.js";
import { configFromUrl } from "./config.js";
import { LoadTiming } from "./loadTiming.js";
import { setupScene, spawnModel } from "./scene.js";

const canvas = document.getElementById("babylon");
const picker = document.getElementById("picker");
const pickerText = document.getElementById("picker-text");

let config;
try {
  config = configFromUrl();
} catch (error) {
  console.error(error.message);
  throw error;
}

// physical pixels, so every machine and browser renders the same amount: the backbuffer is the
// canvas' css size, not scaled by devicePixelRatio
canvas.style.width = `${config.width}px`;
canvas.style.height = `${config.height}px`;
const engine = new Engine(
  canvas,
  // no antialiasing on the canvas, MSAA happens offscreen (scene.js)
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
