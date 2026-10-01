// The benchmark itself, the same as bevy_rendering_benchmark's bench.rs. It starts on its own
// once the model is shown: a warmup that isn't measured, then `runs` runs of `duration` seconds
// each, in which the camera does one full orbit. Every run prints one CSV row, the same columns
// as bencmark_results/fps.csv with `engine,msaa,shadows,camera_distance` in front.

import { COMMON_CSV_HEADER, emitCsv } from "./csv.js";
import { measureModel } from "./measure.js";
import { placeCamera } from "./scene.js";

const FPS_CSV_HEADER =
  "duration_s,frames,avg_fps,median_fps,low_1_fps,low_01_fps,frame_avg_ms,frame_min_ms,frame_p50_ms,frame_p95_ms,frame_p99_ms,frame_max_ms,frame_stddev_ms,mesh_instances,unique_meshes,vertices,triangles,materials,textures,texture_mib,entities";

const TAU = 2 * Math.PI;

function newRun(index) {
  return {
    // 1-based index of this run out of `runs`
    index,
    elapsed: 0,
    frames: [],
    // sanity check independent of the per-frame deltas: every update, including zero-length ones
    updates: 0,
    started: performance.now(),
  };
}

export class Bench {
  constructor(engine, scene, camera, config) {
    this.engine = engine;
    this.scene = scene;
    this.camera = camera;
    this.config = config;
    // loading, warmup, running, done
    this.phase = "loading";
    this.warmupElapsed = 0;
    this.run = null;
    // fps rows of the finished runs, printed together at the end for copying
    this.rows = [];
  }

  /** Once per frame before rendering, `delta` in seconds since the previous frame */
  update(delta, modelShown) {
    const config = this.config;
    if (this.phase === "loading" && modelShown) {
      console.log(
        `Benchmark: ${config.warmup.toFixed(0)} s warmup, then ${config.runs} runs of ${config.duration.toFixed(0)} s`,
      );
      this.phase = "warmup";
      this.warmupElapsed = 0;
    }

    // record_frame_time
    if (this.phase === "warmup") {
      this.warmupElapsed += delta;
      if (this.warmupElapsed >= config.warmup) {
        console.log(`Benchmark run 1/${config.runs} started`);
        this.phase = "running";
        this.run = newRun(1);
      }
    } else if (this.phase === "running") {
      this.run.updates += 1;
      if (delta > 0) {
        this.run.elapsed += delta;
        this.run.frames.push(delta * 1000);
      }
    }

    // orbit_camera, driven by elapsed time so every run covers exactly the same path
    if (this.phase === "warmup") {
      placeCamera(this.camera, (TAU * this.warmupElapsed) / config.duration, config.distance);
    } else if (this.phase === "running") {
      const yaw = TAU * Math.min(this.run.elapsed / config.duration, 1);
      placeCamera(this.camera, yaw, config.distance);
    }

    this.finishRun();
  }

  finishRun() {
    const config = this.config;
    if (this.phase !== "running" || this.run.elapsed < config.duration) return;
    const run = this.run;
    if (run.index < config.runs) {
      console.log(`Benchmark run ${run.index + 1}/${config.runs} started`);
      this.run = newRun(run.index + 1);
    } else {
      this.phase = "done";
      this.run = null;
    }

    const stats = frameStats(run.frames);
    if (stats) {
      const model = measureModel(this.scene);
      const wallClockFps = run.updates / ((performance.now() - run.started) / 1000);
      const summary =
        `\n===== Benchmark run ${run.index}/${config.runs} (${run.elapsed.toFixed(1)} s, one camera orbit) =====\n` +
        `frames            ${stats.frames}\n` +
        `average fps       ${stats.averageFps.toFixed(1)}\n` +
        `median fps        ${stats.medianFps.toFixed(1)}\n` +
        `1% low fps        ${stats.low1Fps.toFixed(1)}\n` +
        `0.1% low fps      ${stats.low01Fps.toFixed(1)}\n` +
        `frame time avg    ${stats.averageMs.toFixed(2)} ms\n` +
        `frame time min    ${stats.minMs.toFixed(2)} ms\n` +
        `frame time p50    ${stats.p50Ms.toFixed(2)} ms\n` +
        `frame time p95    ${stats.p95Ms.toFixed(2)} ms\n` +
        `frame time p99    ${stats.p99Ms.toFixed(2)} ms\n` +
        `frame time max    ${stats.maxMs.toFixed(2)} ms\n` +
        `frame time stddev ${stats.stddevMs.toFixed(2)} ms\n` +
        `wall-clock fps    ${wallClockFps.toFixed(1)} (${run.updates} updates, ${run.updates - stats.frames} zero-length)\n` +
        `mesh instances    ${model.meshInstances}\n` +
        `unique meshes     ${model.uniqueMeshes}\n` +
        `vertices          ${model.vertices}\n` +
        `triangles         ${model.triangles}\n` +
        `materials         ${model.materials}\n` +
        `textures          ${model.textures} (${model.textureMib.toFixed(2)} MiB)\n` +
        `entities          ${model.entities}`;
      const row = emitCsv(
        this.engine,
        config,
        "rendering_fps.csv",
        FPS_CSV_HEADER,
        [
          run.elapsed.toFixed(1),
          stats.frames,
          stats.averageFps.toFixed(1),
          stats.medianFps.toFixed(1),
          stats.low1Fps.toFixed(1),
          stats.low01Fps.toFixed(1),
          stats.averageMs.toFixed(2),
          stats.minMs.toFixed(2),
          stats.p50Ms.toFixed(2),
          stats.p95Ms.toFixed(2),
          stats.p99Ms.toFixed(2),
          stats.maxMs.toFixed(2),
          stats.stddevMs.toFixed(2),
          model.meshInstances,
          model.uniqueMeshes,
          model.vertices,
          model.triangles,
          model.materials,
          model.textures,
          model.textureMib.toFixed(2),
          model.entities,
        ],
        summary,
      );
      this.rows.push(row);
    } else {
      console.warn(`Benchmark run ${run.index}/${config.runs} recorded no frames`);
    }

    if (this.phase === "done") {
      console.log(
        `Benchmark finished, ${config.runs} runs\n----- csv:\n${COMMON_CSV_HEADER},${FPS_CSV_HEADER}\n${this.rows.join("\n")}`,
      );
    }
  }
}

function frameStats(frameTimesMs) {
  if (frameTimesMs.length === 0) return null;
  const sorted = [...frameTimesMs].sort((a, b) => a - b);
  const n = sorted.length;
  const total = sorted.reduce((sum, ms) => sum + ms, 0);
  const averageMs = total / n;
  const variance = sorted.reduce((sum, ms) => sum + (ms - averageMs) ** 2, 0) / n;
  const percentile = (p) => sorted[Math.round((p / 100) * (n - 1))];
  // fps of the average of the slowest `fraction` of frames
  const lowFps = (fraction) => {
    const count = Math.max(Math.ceil(n * fraction), 1);
    const slowest = sorted.slice(n - count);
    return 1000 / (slowest.reduce((sum, ms) => sum + ms, 0) / count);
  };
  const p50Ms = percentile(50);
  return {
    frames: n,
    averageFps: (1000 * n) / total,
    // fps of the typical frame, closer to what the fps overlay shows than `averageFps`
    medianFps: 1000 / p50Ms,
    low1Fps: lowFps(0.01),
    low01Fps: lowFps(0.001),
    averageMs,
    minMs: sorted[0],
    p50Ms,
    p95Ms: percentile(95),
    p99Ms: percentile(99),
    maxMs: sorted[n - 1],
    stddevMs: Math.sqrt(variance),
  };
}
