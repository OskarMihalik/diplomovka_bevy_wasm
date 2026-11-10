// CSV rows, the same columns as bevy_rendering_benchmark, with `engine` = babylon

import { report } from "./report.js";
import { BUILD, browser, cpuCores, csvRow, date, os } from "./systemInfo.js";

/** Columns in front of every CSV row, fps.csv and load.csv start the same without the first three */
export const COMMON_CSV_HEADER =
  "engine,msaa,shadows,camera_distance,date,pc,os,cpu_cores,gpu,graphics_backend,browser,browser_version,build,resolution,model_id,model_name,model_version";

function commonFields(engine, config) {
  const [browserName, browserVersion] = browser();
  return [
    "babylon",
    config.msaa,
    config.shadows ? "on" : "off",
    config.distance.toFixed(2),
    date(),
    config.pc,
    os(),
    cpuCores(),
    gpuName(engine),
    // named like wgpu's backends, so Bevy's and Babylon's rows can be compared
    engine.isWebGPU ? "BrowserWebGpu" : "Gl",
    browserName,
    browserVersion,
    BUILD,
    `${engine.getRenderWidth()}x${engine.getRenderHeight()}`,
    config.modelId,
    config.modelName,
    config.modelVersion,
  ];
}

/**
 * The WebGL renderer, or `vendor architecture` of the WebGPU adapter like Bevy's web build
 * (`intel gen-12lp` in Chrome, empty in Firefox, which hides the adapter info)
 */
function gpuName(engine) {
  if (!engine.isWebGPU) return engine.getGlInfo().renderer;
  const info = engine._adapterInfo ?? {};
  return [info.vendor, info.architecture].filter(Boolean).join(" ");
}

/** Logs `summary` with the CSV row and reports it to the runner, returns the row */
export function emitCsv(engine, config, fileName, header, fields, summary) {
  const row = csvRow([...commonFields(engine, config), ...fields]);
  console.log(`${summary}\n----- csv: ${fileName}\n${COMMON_CSV_HEADER},${header}\n${row}`);
  report({ file: fileName, header: `${COMMON_CSV_HEADER},${header}`, row });
  return row;
}
