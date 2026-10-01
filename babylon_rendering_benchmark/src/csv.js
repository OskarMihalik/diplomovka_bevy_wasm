// CSV rows, the same columns as bevy_rendering_benchmark, with `engine` = babylon

import { BUILD, PC_NAME, browser, cpuCores, csvRow, date, os } from "./systemInfo.js";

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
    PC_NAME,
    os(),
    cpuCores(),
    engine.getGlInfo().renderer,
    // named like wgpu's backends, so Bevy's and Babylon's rows can be compared
    "Gl",
    browserName,
    browserVersion,
    BUILD,
    `${engine.getRenderWidth()}x${engine.getRenderHeight()}`,
    config.modelId,
    config.modelName,
    config.modelVersion,
  ];
}

/** Logs `summary` with the CSV row, returns the row */
export function emitCsv(engine, config, fileName, header, fields, summary) {
  const row = csvRow([...commonFields(engine, config), ...fields]);
  console.log(`${summary}\n----- csv: ${fileName}\n${COMMON_CSV_HEADER},${header}\n${row}`);
  return row;
}
