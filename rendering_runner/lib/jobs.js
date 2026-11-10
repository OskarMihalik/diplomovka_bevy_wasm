// Running one job: a native process or a Firefox started through Playwright. Both return the CSV
// rows the benchmark produced as `{file, header, row}`, and an error if the job failed. The
// rendering benchmarks write rendering_*.csv, the app (app-native, app-web) fps.csv and load.csv.

/** The CSV files a platform writes, `[load, fps]` */
export const csvFiles = (platform) =>
  platform.startsWith("app-") ? ["load.csv", "fps.csv"] : ["rendering_load.csv", "rendering_fps.csv"];

/** The app's bench mode options for a job, see bevy_diplomovka/src/bench_mode.rs */
const appOptions = (job) => ({
  bench_model: String(job.appModel),
  bench_project: String(job.appProject),
  email: ACCOUNT.email,
  password: ACCOUNT.password,
});

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { chromium, firefox } from "playwright";

import { ACCOUNT } from "./app.js";

// launching the browser, opening a page and navigating hang while it can't draw (locked screen)
const STEP_TIMEOUT_MS = 60_000;

// set while a job runs, so Ctrl+C can stop it
let stopCurrent = null;
export const stopRunningJob = () => stopCurrent?.();

/** The time a job may take before it counts as hung */
export const jobTimeoutS = (job) => Math.round(job.estimateS * 1.5 + 120);

/** The name in the CSVs' `browser` column and in meta.json */
const BROWSER_NAMES = { firefox: "Firefox", chrome: "Chrome" };

/**
 * A browser with the same settings for the preflight check and every job: fullscreen without
 * toolbars (the 2088x1550 canvas only fits the screen like this) and an uncapped frame rate.
 * `chrome` is Playwright's Chromium build, its user agent says Chrome. WebGPU is switched on only
 * for `webgpu` jobs, the WebGL2 jobs run in the browser as it was measured before.
 */
function launchBrowser(name, env = {}, api = "webgl2") {
  const common = { headless: false, env: { ...process.env, ...env }, timeout: STEP_TIMEOUT_MS };
  const webgpu = api === "webgpu";
  if (name === "chrome") {
    // both are needed: without --disable-frame-rate-limit Chrome stays at the display's 165 Hz
    // on Wayland, see README.md for what this does to the frame time percentiles
    const args = ["--disable-frame-rate-limit", "--disable-gpu-vsync"];
    // WebGPU on Linux: without these no adapter, with only the first one SwiftShader (CPU)
    if (webgpu) args.push("--enable-unsafe-webgpu", "--enable-features=Vulkan");
    return chromium.launch({ ...common, args });
  }
  const firefoxUserPrefs = { "layout.frame_rate": 0 };
  if (webgpu) {
    // off by default on Linux; requestAdapter fails instead of falling back to a CPU adapter,
    // the only GPU check there is in Firefox, which hides the adapter info
    firefoxUserPrefs["dom.webgpu.enabled"] = true;
    firefoxUserPrefs["dom.webgpu.testing.assert-hardware-adapter"] = true;
    // otherwise every frame is copied back to the CPU to be presented: Bevy barling_wrecks
    // 38 -> 54 fps, Babylon building p99 84 -> 22 ms (PLAN.md section 10)
    firefoxUserPrefs["dom.webgpu.allow-present-without-readback"] = true;
  }
  return firefox.launch({ ...common, args: ["--kiosk"], firefoxUserPrefs });
}

/** A new context and page; Chromium ignores --kiosk for the windows Playwright opens */
async function openPage(browser, name) {
  const context = await within(browser.newContext({ viewport: null }), "creating the browser context");
  const page = await within(context.newPage(), "opening a page");
  if (name === "chrome") {
    const cdp = await context.newCDPSession(page);
    const { windowId } = await cdp.send("Browser.getWindowForTarget");
    await cdp.send("Browser.setWindowBounds", { windowId, bounds: { windowState: "fullscreen" } });
    await cdp.detach();
  }
  return page;
}

const executable = (name) => (name === "chrome" ? chromium : firefox).executablePath();
const installName = (name) => (name === "chrome" ? "chromium" : "firefox");

/** Native Bevy or the native app, writes its CSVs to `jobDir` (`--out`), read back afterwards */
export async function runNative(job, { root, pc, jobDir, log }) {
  fs.rmSync(jobDir, { recursive: true, force: true });
  fs.mkdirSync(jobDir, { recursive: true });
  const app = job.platform === "app-native";
  const binary = path.join(root, "target/release", app ? "bevy_diplomovka" : "bevy_rendering_benchmark");
  const args = [];
  for (const [key, value] of Object.entries(app ? appOptions(job) : { model: job.model })) args.push(`--${key}`, value);
  for (const [key, value] of Object.entries(job.options)) {
    // the app picks the GPU through wgpu's environment variable
    if (!["out", "exit", "report", "pc", ...(app ? ["gpu"] : [])].includes(key)) args.push(`--${key}`, value);
  }
  args.push("--pc", pc, "--out", jobDir, "--exit", "on");
  log.write(`$ ${binary} ${args.join(" ").replace(ACCOUNT.password, "***")}\n`);

  const env = { ...process.env };
  if (app) {
    // the binary runs from target/release, its assets are next to Cargo.toml
    env.BEVY_ASSET_ROOT = path.join(root, "bevy_diplomovka");
    if (job.options.gpu) env.WGPU_POWER_PREF = job.options.gpu;
  }
  const child = spawn(binary, args, { env: { ...env, ...job.env }, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout.on("data", (data) => log.write(data));
  child.stderr.on("data", (data) => log.write(data));
  const timeoutS = jobTimeoutS(job);
  const outcome = await new Promise((resolve) => {
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      resolve(`timeout after ${timeoutS} s`);
    }, timeoutS * 1000);
    stopCurrent = () => {
      child.kill("SIGKILL");
      resolve("stopped");
    };
    child.on("error", (error) => resolve(error.message));
    child.on("exit", (code, signal) => {
      clearTimeout(timer);
      resolve(code === 0 ? null : `exited with ${code ?? signal}`);
    });
  });
  stopCurrent = null;

  const rows = [];
  for (const file of csvFiles(job.platform)) {
    const csvPath = path.join(jobDir, file);
    if (!fs.existsSync(csvPath)) continue;
    const [header, ...lines] = fs.readFileSync(csvPath, "utf8").split("\n").filter(Boolean);
    for (const row of lines) rows.push({ file, header, row });
  }
  return { rows, error: outcome };
}

/** Bevy, Babylon or the app in a new browser with a new profile, the page posts its rows */
export async function runWeb(job, { server, pc, log }) {
  // the url path the server mounts each dist at
  const engine = job.platform.replace(/-web$/, "");
  const webgpu = job.api === "webgpu";
  const rows = [];
  let settle;
  const finished = new Promise((resolve) => (settle = resolve));
  server.model = job.model;
  server.onReport = (message) => {
    log.write(`[report] ${JSON.stringify(message)}\n`);
    if (message.file) rows.push({ file: message.file, header: message.header, row: message.row });
    if (message.done) settle(null);
    if (message.error) settle(message.error);
  };
  stopCurrent = () => settle("stopped");

  let browser;
  let timer;
  let error;
  try {
    browser = await launchBrowser(job.browser, job.env, job.api);
    log.write(`${job.browser} ${browser.version()}\n`);
    browser.on("disconnected", () => settle(`${BROWSER_NAMES[job.browser]} closed`));
    const page = await openPage(browser, job.browser);
    page.on("console", (message) => log.write(`[console.${message.type()}] ${message.text()}\n`));
    // a config error, a wasm trap (e.g. out of memory)
    page.on("pageerror", (pageError) => settle(`page error: ${pageError.message}`));
    page.on("crash", () => settle("page crashed"));

    const params = new URLSearchParams({
      ...job.options,
      // the app downloads the model from the backend, the rendering benchmarks from this server
      ...(engine === "app" ? appOptions(job) : { model: `/models/${path.basename(job.model)}` }),
      // Bevy's WebGPU is its own build, Babylon picks the engine from the url
      ...(webgpu && engine === "babylon" ? { api: "webgpu" } : {}),
      report: "/report",
      pc,
    });
    const url = `${server.origin}/${engine}${webgpu && engine === "bevy" ? "-webgpu" : ""}/?${params}`;
    log.write(`open ${url.replace(ACCOUNT.password, "***")}\n`);
    await within(page.goto(url, { waitUntil: "commit" }), "opening the benchmark page");

    const timeoutS = jobTimeoutS(job);
    timer = setTimeout(() => settle(`timeout after ${timeoutS} s`), timeoutS * 1000);
    error = await finished;
  } catch (thrown) {
    error = thrown.message.split("\n")[0];
  } finally {
    clearTimeout(timer);
    stopCurrent = null;
    server.onReport = () => {};
    await browser?.close().catch(() => {});
  }
  return { rows, error, browserVersion: browser?.version() };
}

/**
 * Opens an empty page once before the batch: the browser starts, WebGL2 runs on a GPU, the page
 * is cross-origin isolated (20 µs timer steps in Firefox, 5 µs in Chrome). Returns what goes into
 * meta.json.
 */
export async function checkBrowser(name, server, env = {}, api = "webgl2") {
  if (!fs.existsSync(executable(name))) {
    throw new Error(
      `Playwright's ${installName(name)} isn't installed: cd rendering_runner && npx playwright install ${installName(name)}`,
    );
  }
  const browser = await launchBrowser(name, env, api);
  try {
    const page = await openPage(browser, name);
    await within(page.goto(`${server.origin}/check`), "opening the check page");
    const info = await page.evaluate(async (webgpu) => {
      let adapter = null;
      if (webgpu) {
        const found = await navigator.gpu?.requestAdapter({ powerPreference: "low-power" }).catch(() => null);
        adapter = found
          ? { vendor: found.info.vendor, architecture: found.info.architecture, fallback: Boolean(found.info.isFallbackAdapter) }
          : null;
      }
      const gl = document.createElement("canvas").getContext("webgl2");
      // Chrome's RENDERER is "WebKit WebGL", the GPU is only in the unmasked one
      const debug = gl?.getExtension("WEBGL_debug_renderer_info");
      let timerStepMs = Infinity;
      let last = performance.now();
      for (let i = 0; i < 100000; i++) {
        const now = performance.now();
        if (now > last) timerStepMs = Math.min(timerStepMs, now - last);
        last = now;
      }
      return {
        webgl2_renderer: gl ? gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER) : null,
        cross_origin_isolated: crossOriginIsolated,
        timer_step_ms: Number(timerStepMs.toFixed(4)),
        screen: `${screen.width}x${screen.height}`,
        device_pixel_ratio: devicePixelRatio,
        window: `${innerWidth}x${innerHeight}`,
        user_agent: navigator.userAgent,
        ...(webgpu ? { webgpu_adapter: adapter } : {}),
      };
    }, api === "webgpu");
    return { browser: BROWSER_NAMES[name], version: browser.version(), ...info };
  } finally {
    await browser.close().catch(() => {});
  }
}

function within(promise, what) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(
      () => reject(new Error(`${what} took longer than ${STEP_TIMEOUT_MS / 1000} s (is the screen locked?)`)),
      STEP_TIMEOUT_MS,
    );
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}
