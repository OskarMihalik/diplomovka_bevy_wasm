// How long opening the model takes, split into the same steps as bevy_rendering_benchmark's
// load_timing.rs: download + parsing (until every texture is decoded), mesh creation (adding the
// model to the scene) and the first frame (until the frame after that has started).

import { emitCsv } from "./csv.js";
import { meshCounts } from "./measure.js";

const LOAD_CSV_HEADER =
  "meshes,vertices,triangles,download_parse_ms,mesh_creation_ms,first_frame_ms,total_ms,frames";

export class LoadTiming {
  /** Started when the model is requested, at startup or once it was picked on the page */
  constructor(frame) {
    this.started = performance.now();
    this.startedFrame = frame;
    // end of the previous step
    this.lastStep = this.started;
    this.downloadParseMs = 0;
    this.spawnMs = 0;
    // frame in which the model was added to the scene
    this.spawnedFrame = null;
  }

  /** How long the step that just finished took, in ms */
  step() {
    const now = performance.now();
    const step = now - this.lastStep;
    this.lastStep = now;
    return step;
  }

  parsed() {
    this.downloadParseMs = this.step();
    console.log(`model load: download + parsing took ${this.downloadParseMs.toFixed(1)} ms`);
  }

  spawned(frame) {
    this.spawnMs = this.step();
    this.spawnedFrame = frame;
    console.log(`model load: mesh creation (scene spawn) took ${this.spawnMs.toFixed(1)} ms`);
  }

  /** Called at the start of every frame, true once the model has been rendered and logged */
  finish(frame, model, engine, config) {
    // the frame the model was added in has been rendered once the next frame starts
    if (this.spawnedFrame === null || frame <= this.spawnedFrame) return false;
    const firstFrameMs = this.step();

    let meshes = 0;
    let vertices = 0;
    let triangles = 0;
    for (const mesh of model.getChildMeshes(false)) {
      const [v, t] = meshCounts(mesh);
      if (v === 0) continue;
      meshes += 1;
      vertices += v;
      triangles += t;
    }
    const totalMs = performance.now() - this.started;
    const frames = frame - this.startedFrame;

    const pad = (value, digits) =>
      (digits === undefined ? String(value) : value.toFixed(digits)).padStart(9);
    const summary =
      `\n===== model load: ${config.modelName} =====\n` +
      `download + parsing  ${pad(this.downloadParseMs, 1)} ms\n` +
      `mesh creation       ${pad(this.spawnMs, 1)} ms\n` +
      `first frame render  ${pad(firstFrameMs, 1)} ms\n` +
      `total               ${pad(totalMs, 1)} ms\n` +
      `meshes              ${pad(meshes)}\n` +
      `vertices            ${pad(vertices)}\n` +
      `triangles           ${pad(triangles)}\n` +
      `frames until shown  ${pad(frames)}`;
    emitCsv(
      engine,
      config,
      "rendering_load.csv",
      LOAD_CSV_HEADER,
      [
        meshes,
        vertices,
        triangles,
        this.downloadParseMs.toFixed(1),
        this.spawnMs.toFixed(1),
        firstFrameMs.toFixed(1),
        totalMs.toFixed(1),
        frames,
      ],
      summary,
    );
    return true;
  }
}
