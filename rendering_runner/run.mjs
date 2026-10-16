// Runs the rendering and loading benchmarks from a matrix CSV and appends the rows to
// rendering_fps.csv and rendering_load.csv. See README.md and PLAN.md.
//
//   node rendering_runner/run.mjs <matrix.csv> [--pc NAME] [--out DIR] [--models DIR]
//       [--cooldown 10] [--no-build] [--dry-run] [--resume BATCH] [--port 8090]

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { busyApps, keepScreenAwake, machineInfo, onAcPower, screenLocked } from "./lib/host.js";
import { checkFirefox, jobTimeoutS, runNative, runWeb, stopRunningJob } from "./lib/jobs.js";
import { formatDuration, parseCsv, readMatrix } from "./lib/matrix.js";
import { startServer } from "./lib/server.js";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const USAGE = `usage: node rendering_runner/run.mjs <matrix.csv> [--pc NAME] [--out DIR] [--models DIR]
    [--cooldown 10] [--no-build] [--dry-run] [--resume BATCH] [--port 8090]`;
// a job that ran while the screen was locked is repeated, at most this often
const LOCKED_RETRIES = 2;

// --- arguments --------------------------------------------------------------------------------
const args = process.argv.slice(2);
const flag = (name) => {
  const index = args.indexOf(`--${name}`);
  if (index < 0) return false;
  args.splice(index, 1);
  return true;
};
const option = (name, fallback) => {
  const index = args.indexOf(`--${name}`);
  if (index < 0) return fallback;
  const [, value] = args.splice(index, 2);
  if (value === undefined) fail(`missing value for --${name}`);
  return value;
};
function fail(message) {
  console.error(`${message}\n${USAGE}`);
  process.exit(2);
}

const dryRun = flag("dry-run");
const noBuild = flag("no-build");
const pc = option("pc", os.hostname());
const outDir = path.resolve(option("out", path.join(ROOT, "bencmark_results")));
const modelsDir = path.resolve(option("models", path.join(os.homedir(), "Documents/diplomovka_models")));
const cooldownS = Number(option("cooldown", "10"));
const resume = option("resume", null);
const port = Number(option("port", "8090"));
if (args.length !== 1 || args[0].startsWith("--")) fail(args.length ? `unexpected ${args.join(" ")}` : "missing matrix.csv");
const matrixFile = path.resolve(args[0]);

let jobs;
try {
  jobs = readMatrix(matrixFile, modelsDir);
} catch (error) {
  console.error(error.message);
  process.exit(2);
}
const platforms = new Set(jobs.map((job) => job.platform));
const hasWeb = [...platforms].some((platform) => platform.endsWith("-web"));

// --- batch folder, --resume -------------------------------------------------------------------
const logsDir = path.join(outDir, "logs/rendering");
const batch = resume ?? new Date().toISOString().slice(0, 19).replace(/[-:]/g, "").replace("T", "-");
const batchDir = path.join(logsDir, batch);
const done = new Set();
if (resume) {
  if (!fs.existsSync(batchDir)) fail(`no batch ${batchDir}`);
  const previous = fs.readFileSync(path.join(batchDir, "matrix.csv"), "utf8");
  if (previous !== fs.readFileSync(matrixFile, "utf8")) {
    console.error(`${matrixFile} differs from the batch's matrix.csv, resume with the same matrix`);
    process.exit(2);
  }
  const doneFile = path.join(batchDir, "done.txt");
  if (fs.existsSync(doneFile)) {
    for (const id of fs.readFileSync(doneFile, "utf8").split("\n").filter(Boolean)) done.add(id);
  }
}
const todo = jobs.filter((job) => !done.has(job.id));
const cooldownTotal = Math.max(todo.length - 1, 0) * cooldownS;
const estimate = (list) => list.reduce((sum, job) => sum + job.estimateS, 0);

console.log(
  `${jobs.length} jobs${done.size ? `, ${done.size} done before, ${todo.length} left` : ""}, ` +
    `about ${formatDuration(estimate(todo) + cooldownTotal)}`,
);
if (dryRun) {
  for (const job of todo) {
    const options = Object.entries(job.options).map(([key, value]) => `${key}=${value}`).join(" ");
    const env = Object.keys(job.env).length ? ` env ${Object.entries(job.env).map(([k, v]) => `${k}=${v}`).join(";")}` : "";
    console.log(`  ${job.id.padEnd(60)} ${formatDuration(job.estimateS).padStart(8)}  ${options}${env}`);
  }
  process.exit(0);
}

// --- checks before starting -------------------------------------------------------------------
const warnings = [];
if (onAcPower() === false) warnings.push("the laptop is on battery");
const apps = busyApps();
if (apps.length) warnings.push(`close these apps, they share the GPU and CPU: ${apps.join(", ")}`);
if (screenLocked()) warnings.push("the screen is locked, jobs wait until it is unlocked");
for (const warning of warnings) console.warn(`warning: ${warning}`);

fs.mkdirSync(batchDir, { recursive: true });
if (!resume) fs.copyFileSync(matrixFile, path.join(batchDir, "matrix.csv"));
const shortPath = (file) => (path.relative(process.cwd(), file).startsWith("..") ? file : path.relative(process.cwd(), file));
console.log(`logs: ${shortPath(batchDir)}`);

// --- build ------------------------------------------------------------------------------------
if (!noBuild) {
  const buildLog = path.join(batchDir, "build.log");
  const steps = [];
  if (platforms.has("bevy-native")) {
    steps.push(["cargo", ["build", "-p", "bevy_rendering_benchmark", "--release"], ROOT]);
  }
  if (platforms.has("bevy-web")) {
    steps.push([
      "trunk",
      ["build", "--release", "--cargo-profile", "wasm-release"],
      path.join(ROOT, "bevy_rendering_benchmark"),
    ]);
  }
  if (platforms.has("babylon-web")) {
    steps.push(["npm", ["run", "build"], path.join(ROOT, "babylon_rendering_benchmark")]);
  }
  for (const [command, commandArgs, cwd] of steps) {
    console.log(`build: ${command} ${commandArgs.join(" ")}`);
    try {
      const output = execFileSync(command, commandArgs, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], maxBuffer: 1 << 28 });
      fs.appendFileSync(buildLog, `$ ${command} ${commandArgs.join(" ")}\n${output}\n`);
    } catch (error) {
      fs.appendFileSync(buildLog, `$ ${command} ${commandArgs.join(" ")}\n${error.stdout ?? ""}${error.stderr ?? ""}\n`);
      console.error(`build failed, see ${buildLog}`);
      process.exit(1);
    }
  }
}

// --- meta.json --------------------------------------------------------------------------------
const metaFile = path.join(batchDir, "meta.json");
const meta = fs.existsSync(metaFile) ? JSON.parse(fs.readFileSync(metaFile, "utf8")) : { runs: [] };
const thisRun = {
  started: new Date().toISOString(),
  pc,
  matrix: path.relative(ROOT, matrixFile),
  models: modelsDir,
  out: outDir,
  playwright: JSON.parse(fs.readFileSync(path.join(ROOT, "rendering_runner/node_modules/playwright/package.json"), "utf8")).version,
  machine: machineInfo(ROOT),
  warnings,
};
meta.runs.push(thisRun);
const writeMeta = () => fs.writeFileSync(metaFile, JSON.stringify(meta, null, 2) + "\n");
writeMeta();

// --- server, Firefox check, keeping the screen awake ------------------------------------------
let server = null;
if (hasWeb) {
  server = await startServer(
    {
      "/bevy/": path.join(ROOT, "bevy_rendering_benchmark/dist"),
      "/babylon/": path.join(ROOT, "babylon_rendering_benchmark/dist"),
    },
    port,
  );
}
const stopKeepingAwake = keepScreenAwake();
let interrupted = false;
process.on("SIGINT", () => {
  if (interrupted) process.exit(130);
  interrupted = true;
  console.log("\nstopping after the current job is killed, Ctrl+C again to quit at once");
  stopRunningJob();
});

if (hasWeb) {
  await waitForUnlock();
  try {
    thisRun.firefox = await checkFirefox(server);
  } catch (error) {
    console.error(`Firefox check failed: ${error.message}`);
    await finish(1);
  }
  writeMeta();
  const check = thisRun.firefox;
  console.log(`firefox ${check.firefox}, WebGL2 ${check.webgl2_renderer}, timer step ${check.timer_step_ms} ms`);
  if (!check.webgl2_renderer) {
    console.error("Firefox has no WebGL2");
    await finish(1);
  }
  if (!check.cross_origin_isolated) console.warn("warning: the page isn't cross-origin isolated, frame times are rounded to 1 ms");
}

// --- jobs -------------------------------------------------------------------------------------
const failed = [];
let passed = 0;
for (const [index, job] of todo.entries()) {
  if (interrupted) break;
  const left = estimate(todo.slice(index)) + Math.max(todo.length - index - 1, 0) * cooldownS;
  const options = Object.entries(job.options).map(([key, value]) => `${key}=${value}`).join(" ");
  console.log(`[${index + 1}/${todo.length}] ${job.id}  ${options}  (${formatDuration(left)} left)`);

  const log = fs.createWriteStream(path.join(batchDir, `${job.id}.log`), { flags: "a" });
  let result;
  for (let attempt = 0; ; attempt++) {
    await waitForUnlock();
    const started = Date.now();
    log.write(`\n===== ${new Date().toISOString()} attempt ${attempt + 1}, timeout ${jobTimeoutS(job)} s\n`);
    result =
      job.platform === "bevy-native"
        ? await runNative(job, { root: ROOT, pc, jobDir: path.join(batchDir, "native", job.id), log })
        : await runWeb(job, { server, pc, log });
    result.seconds = Math.round((Date.now() - started) / 1000);
    result.error ??= validate(job, result.rows);
    // a locked screen stops Firefox from drawing and changes native frame times
    if (screenLocked() && !interrupted) {
      result.error = `the screen was locked${result.error ? ` (${result.error})` : ""}`;
      if (attempt < LOCKED_RETRIES) {
        console.log(`    ${result.error}, repeating after the unlock`);
        continue;
      }
    }
    break;
  }
  log.end();

  if (result.firefox && !thisRun.firefox_versions?.includes(result.firefox)) {
    thisRun.firefox_versions = [...(thisRun.firefox_versions ?? []), result.firefox];
  }
  if (result.error) {
    failed.push({ id: job.id, error: result.error });
    fs.appendFileSync(path.join(batchDir, "failed.txt"), `${job.id}\t${result.error}\n`);
    console.log(`    FAILED after ${result.seconds} s: ${result.error}`);
  } else {
    try {
      appendRows(result.rows);
    } catch (error) {
      console.error(error.message);
      await finish(1);
    }
    fs.appendFileSync(path.join(batchDir, "done.txt"), `${job.id}\n`);
    passed += 1;
    console.log(`    ok after ${result.seconds} s: ${describe(result.rows)}`);
  }
  if (!interrupted && index < todo.length - 1) await sleep(cooldownS * 1000);
}

thisRun.passed = passed;
thisRun.failed = failed;
console.log(
  `\n${passed} passed, ${failed.length} failed${interrupted ? ", interrupted" : ""}` +
    (interrupted || failed.length ? `; resume with --resume ${batch}` : ""),
);
for (const { id, error } of failed) console.log(`  ${id}: ${error}`);
await finish(failed.length || interrupted ? 1 : 0);

// --- helpers ----------------------------------------------------------------------------------
async function finish(code) {
  thisRun.finished = new Date().toISOString();
  writeMeta();
  stopKeepingAwake();
  await server?.close();
  process.exit(code);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForUnlock() {
  if (!screenLocked()) return;
  console.log("    the screen is locked, waiting for the unlock");
  while (screenLocked() && !interrupted) await sleep(5000);
}

/** null if the job produced what it should: one load row, `runs` fps rows, the expected GPU */
function validate(job, rows) {
  const count = (file) => rows.filter((row) => row.file === file).length;
  if (count("rendering_load.csv") !== 1) return `${count("rendering_load.csv")} load rows instead of 1`;
  if (count("rendering_fps.csv") !== job.runs) return `${count("rendering_fps.csv")} fps rows instead of ${job.runs}`;
  if (job.expectGpu) {
    for (const { header, row } of rows) {
      const gpu = field(header, row, "gpu");
      if (!gpu.toLowerCase().includes(job.expectGpu.toLowerCase())) {
        return `ran on \`${gpu}\`, expected \`${job.expectGpu}\``;
      }
    }
  }
  return null;
}

/** Appends to <out>/<file>, the header only into a new file; a different header stops the batch */
function appendRows(rows) {
  for (const { file, header, row } of rows) {
    const csvPath = path.join(outDir, file);
    const existing = fs.existsSync(csvPath) ? fs.readFileSync(csvPath, "utf8").split("\n", 1)[0] : "";
    if (existing && existing !== header) {
      throw new Error(`${csvPath} has other columns than the benchmark writes:\n  file:      ${existing}\n  benchmark: ${header}`);
    }
    fs.appendFileSync(csvPath, `${existing ? "" : `${header}\n`}${row}\n`);
  }
}

function field(header, row, name) {
  const [columns] = parseCsv(header);
  const [values] = parseCsv(row);
  return values[columns.indexOf(name)] ?? "";
}

/** `load 4.3 s, 62.1 fps` */
function describe(rows) {
  const parts = [];
  for (const { file, header, row } of rows) {
    if (file === "rendering_load.csv") parts.push(`load ${(Number(field(header, row, "total_ms")) / 1000).toFixed(1)} s`);
  }
  const fps = rows.filter((row) => row.file === "rendering_fps.csv").map((r) => field(r.header, r.row, "avg_fps"));
  if (fps.length) parts.push(`${fps.join(", ")} fps`);
  return parts.join(", ");
}
