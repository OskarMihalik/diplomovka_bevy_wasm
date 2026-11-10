// The matrix CSV: one row per benchmark configuration, expanded into jobs. See PLAN.md, section 2.

import fs from "node:fs";
import path from "node:path";

// app-*: the whole bevy_diplomovka app in bench mode (PLAN.md section 7)
export const PLATFORMS = ["bevy-native", "bevy-web", "babylon-web", "app-native", "app-web"];
// the web platforms run in Playwright's Firefox or Chromium (written as Chrome in the CSVs)
export const BROWSERS = ["firefox", "chrome"];
// the web platforms' graphics API, webgpu is a second Bevy build (dist-webgpu), Babylon's `api`
export const APIS = ["webgl2", "webgpu"];
// columns only the web platforms use
const WEB_COLUMNS = ["browser", "api"];

// columns the runner uses itself, every other column is a benchmark option
const RUNNER_COLUMNS = ["platform", "model", "repeat", "env", "expect_gpu", ...WEB_COLUMNS];
// never split on spaces
const NO_EXPANSION = ["env", "expect_gpu", "name"];
// the benchmarks' defaults, for the time estimate and the expected number of fps rows
const DEFAULTS = { runs: 5, duration: 60, warmup: 5 };
// download, parsing, first frame and closing, a guess for the estimate
const OVERHEAD_S = 20;

/** Splits CSV text into rows of fields, `"` quoting as in RFC 4180 */
export function parseCsv(text) {
  const rows = [];
  let row = [];
  let field = "";
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"' && text[i + 1] === '"') {
        field += '"';
        i++;
      } else if (c === '"') {
        quoted = false;
      } else {
        field += c;
      }
    } else if (c === '"') {
      quoted = true;
    } else if (c === ",") {
      row.push(field);
      field = "";
    } else if (c === "\n" || c === "\r") {
      if (c === "\r" && text[i + 1] === "\n") i++;
      row.push(field);
      rows.push(row);
      row = [];
      field = "";
    } else {
      field += c;
    }
  }
  if (field !== "" || row.length > 0) {
    row.push(field);
    rows.push(row);
  }
  return rows;
}

/**
 * Reads the matrix and returns the jobs in the order they run: repetition 1 of every row, then
 * repetition 2 of every row, ... Lines starting with `#` and empty lines are skipped.
 */
export function readMatrix(file, modelsDir) {
  const lines = fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((line) => line.trim() !== "" && !line.trimStart().startsWith("#"));
  const [header, ...rows] = parseCsv(lines.join("\n"));
  const columns = header.map((column) => column.trim());
  for (const required of ["platform", "model"]) {
    if (!columns.includes(required)) throw new Error(`${file}: no \`${required}\` column`);
  }

  const configs = [];
  rows.forEach((fields, index) => {
    const line = `${file} row ${index + 1}`;
    const cells = Object.fromEntries(columns.map((column, i) => [column, (fields[i] ?? "").trim()]));
    // every combination of the space-separated values
    let combinations = [{}];
    for (const [column, cell] of Object.entries(cells)) {
      if (cell === "") continue;
      const values = NO_EXPANSION.includes(column) ? [cell] : cell.split(/\s+/);
      combinations = combinations.flatMap((combination) =>
        values.map((value) => ({ ...combination, [column]: value })),
      );
    }
    // native platforms have no browser or api: one job, not one per value of those cells
    const first = Object.fromEntries(WEB_COLUMNS.map((column) => [column, (cells[column] ?? "").split(/\s+/)[0]]));
    combinations = combinations.filter(
      (combination) =>
        combination.platform?.endsWith("-web") ||
        WEB_COLUMNS.every((column) => (combination[column] ?? "") === first[column]),
    );
    for (const cells of combinations) configs.push(config(cells, line, modelsDir));
  });

  const jobs = [];
  const rounds = Math.max(0, ...configs.map((config) => config.repeat));
  for (let rep = 1; rep <= rounds; rep++) {
    configs.forEach((config, index) => {
      if (rep > config.repeat) return;
      // the defaults aren't in the id, so batches from before these columns still resume
      const browser = config.browser && config.browser !== "firefox" ? `-${config.browser}` : "";
      const api = config.api === "webgpu" ? "-webgpu" : "";
      const id = `${String(index + 1).padStart(3, "0")}-${config.platform}${browser}${api}-${config.modelName}-rep${rep}`;
      jobs.push({ ...config, id, rep });
    });
  }
  return jobs;
}

function config(cells, line, modelsDir) {
  const platform = cells.platform;
  if (!PLATFORMS.includes(platform)) {
    throw new Error(`${line}: platform must be one of ${PLATFORMS.join(", ")}, got \`${platform}\``);
  }
  const model = modelPath(cells.model, modelsDir);
  if (!fs.existsSync(model)) throw new Error(`${line}: model not found: ${model}`);
  const web = platform.endsWith("-web");
  const browser = web ? cells.browser ?? "firefox" : null;
  if (web && !BROWSERS.includes(browser)) {
    throw new Error(`${line}: browser must be one of ${BROWSERS.join(", ")}, got \`${browser}\``);
  }
  const api = web ? cells.api ?? "webgl2" : null;
  if (web && !APIS.includes(api)) {
    throw new Error(`${line}: api must be one of ${APIS.join(", ")}, got \`${api}\``);
  }
  if (platform === "app-web" && api === "webgpu") {
    throw new Error(`${line}: the app has no WebGPU build, api=webgpu only works for bevy-web and babylon-web`);
  }
  const repeat = Number(cells.repeat ?? 1);
  if (!Number.isInteger(repeat) || repeat < 1) {
    throw new Error(`${line}: repeat must be a whole number from 1, got \`${cells.repeat}\``);
  }
  const env = {};
  for (const pair of (cells.env ?? "").split(";").filter(Boolean)) {
    const [key, ...value] = pair.split("=");
    if (!key || value.length === 0) throw new Error(`${line}: env must look like A=1;B=2, got \`${cells.env}\``);
    env[key.trim()] = value.join("=").trim();
  }
  const options = Object.fromEntries(
    Object.entries(cells).filter(([column]) => !RUNNER_COLUMNS.includes(column)),
  );
  const number = (key) => {
    const value = Number(options[key] ?? DEFAULTS[key]);
    if (Number.isNaN(value)) throw new Error(`${line}: ${key} must be a number, got \`${options[key]}\``);
    return value;
  };
  const runs = Math.max(Math.trunc(number("runs")), 0);
  return {
    platform,
    browser,
    api,
    model,
    modelName: path.basename(model).replace(/\.(glb|gltf)$/, ""),
    repeat,
    env,
    expectGpu: cells.expect_gpu ?? "",
    options,
    runs,
    estimateS: OVERHEAD_S + (runs === 0 ? 0 : number("warmup") + runs * number("duration")),
  };
}

/** A path, or a name in the models folder with or without `.glb` */
function modelPath(model, modelsDir) {
  if (!model) return "";
  if (model.includes("/")) return path.resolve(model.replace(/^~(?=\/)/, process.env.HOME));
  return path.join(modelsDir, /\.(glb|gltf)$/.test(model) ? model : `${model}.glb`);
}

/** `45 s`, `12 min`, `1 h 05 min` */
export function formatDuration(seconds) {
  if (seconds < 90) return `${Math.round(seconds)} s`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, "0")} min`;
}
