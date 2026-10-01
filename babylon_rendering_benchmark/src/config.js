// Benchmark settings from the url query, the same options as bevy_rendering_benchmark.
//
//   http://localhost:8083/?gpu=low                 then pick or drop the model on the page
//   http://localhost:8083/?model=http://localhost:8090/14.glb&gpu=low
//
// - `model`      url of a .glb, relative to the page; without it the model is picked on the page
// - `id`         model id for the CSV, defaults to the file name if it is a number (`14.glb`)
// - `name`       model name for the CSV, defaults to the name of a known id, else the file name
// - `version`    model version for the CSV, defaults to 1
// - `runs`       how many measured runs, default 5
// - `duration`   seconds of one run, the camera does one full orbit in this time, default 60
// - `warmup`     seconds before the first run that are not measured, default 5
// - `gpu`        `low` or `high`, the WebGL power preference, default high
// - `resolution` canvas size in physical pixels, `2088x1550` by default
// - `msaa`       1, 2, 4 or 8 samples, default 4 like bevy_diplomovka
// - `shadows`    `on` or `off`, shadows of the point light, default on like bevy_diplomovka
// - `distance`   meters from the camera to the center of the orbit, default 5.22 like
//                bevy_diplomovka, the model is scaled to 0.25 so a 100 m model is 25 m wide

import { DEFAULT_DISTANCE } from "./scene.js";

// Known models, so the CSV rows can be joined with bencmark_results/fps.csv by id
const MODELS = [
  [12, "Barling Wrecks", 1],
  [13, "King's hall", 1],
  [14, "Cathedral", 1],
  [15, "Hong Kong City Buiildings", 1],
  [16, "Acrisure Stadium", 1],
  [17, "Cei Mep Port", 1],
  [18, "Administrative and Warehouse Comples", 1],
];

/** Settings from the page url, `model` replaces the one in the url (a picked file's name) */
export function configFromUrl(model) {
  const params = new URLSearchParams(location.search);
  if (model !== undefined) {
    params.set("model", model);
  } else if (params.get("model")) {
    params.set("model", new URL(params.get("model"), location.href).href);
  }
  return parse(params);
}

function parse(params) {
  const get = (key) => params.get(key) ?? undefined;
  const number = (key, fallback) => {
    const value = get(key);
    if (value === undefined) return fallback;
    const parsed = Number(value);
    if (value === "" || Number.isNaN(parsed)) {
      throw new Error(`\`${key}\` must be a number, got \`${value}\``);
    }
    return parsed;
  };
  const onOff = (key, fallback) => {
    const value = get(key);
    if (value === undefined) return fallback;
    if (["on", "true", "1"].includes(value)) return true;
    if (["off", "false", "0"].includes(value)) return false;
    throw new Error(`\`${key}\` must be \`on\` or \`off\`, got \`${value}\``);
  };

  // empty means it is picked on the page
  const model = get("model") ?? "";
  const stem = model
    .split(/[/\\]/)
    .pop()
    .split(/[?#]/)[0]
    .replace(/\.glb$/, "")
    .replace(/\.gltf$/, "");
  const modelId = get("id") ?? (/^\d+$/.test(stem) ? stem : "");
  const known = MODELS.find(([id]) => String(id) === modelId);
  const modelName = get("name") ?? (known ? known[1] : stem);
  const modelVersion = get("version") ?? (known ? String(known[2]) : "1");

  const gpu = get("gpu") ?? "high";
  if (gpu !== "low" && gpu !== "high") {
    throw new Error(`\`gpu\` must be \`low\` or \`high\`, got \`${gpu}\``);
  }
  const resolution = get("resolution") ?? "2088x1550";
  const match = /^(\d+)x(\d+)$/.exec(resolution);
  if (!match) {
    throw new Error(`\`resolution\` must look like 1920x1080, got \`${resolution}\``);
  }
  const msaa = Math.trunc(number("msaa", 4));
  if (![1, 2, 4, 8].includes(msaa)) {
    throw new Error(`\`msaa\` must be 1, 2, 4 or 8, got ${msaa}`);
  }

  return {
    model,
    modelId,
    modelName,
    modelVersion,
    runs: Math.max(Math.trunc(number("runs", 5)), 1),
    duration: Math.max(number("duration", 60), 1),
    warmup: Math.max(number("warmup", 5), 0),
    powerPreference: gpu === "low" ? "low-power" : "high-performance",
    width: Number(match[1]),
    height: Number(match[2]),
    msaa,
    shadows: onOff("shadows", true),
    distance: Math.max(number("distance", DEFAULT_DISTANCE), 0.01),
  };
}
