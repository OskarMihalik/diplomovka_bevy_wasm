// Prepares the backend for the bevy_diplomovka app benchmark (PLAN.md, section 7), only through
// its API: the benchmark account, its project and the models of the matrix. Database ids differ
// between PCs, so the matrix names models by file name and the ids are looked up here.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export const ACCOUNT = { email: "bench@bench.local", username: "bench", password: "bench-password" };
const PROJECT = "benchmarks";

/** The backend url the app was built with, FRONTEND_BACKEND_URL in the repository's .env */
export function backendUrl(root) {
  const env = fs.readFileSync(path.join(root, ".env"), "utf8");
  const match = /^FRONTEND_BACKEND_URL=(.*)$/m.exec(env);
  if (!match) throw new Error(`${root}/.env has no FRONTEND_BACKEND_URL`);
  return match[1].trim().replace(/\/$/, "");
}

/**
 * Starts Postgres and the backend if needed, logs in (or registers) the benchmark account, makes
 * sure its project has every model of `modelFiles` (uploaded from disk unless it is there and its
 * file downloads). Returns `{ project, models: Map<file name, model id> }`.
 */
export async function prepareBackend(root, backend, modelFiles, log) {
  await ensureBackend(root, backend, log);
  const token = await logIn(backend, log);
  const api = async (method, route, body) => {
    const response = await fetch(`${backend}${route}`, {
      method,
      headers: { authorization: token, ...(body && !(body instanceof FormData) ? { "content-type": "application/json" } : {}) },
      body: body instanceof FormData ? body : body && JSON.stringify(body),
    });
    return unwrap(await response.json(), `${method} ${route}`);
  };

  let project = (await api("GET", "/project")).find((p) => p.name === PROJECT);
  if (!project) {
    log(`creating project "${PROJECT}"`);
    const projects = await api("POST", "/project", { name: PROJECT, description: "rendering_runner" });
    project = projects.find((p) => p.name === PROJECT);
    if (!project) throw new Error(`project "${PROJECT}" wasn't created`);
  }

  const models = new Map();
  for (const file of new Set(modelFiles)) {
    const name = path.basename(file).replace(/\.(glb|gltf)$/, "");
    // newest first, a model whose file is gone (recreated container) is skipped
    const existing = (await api("GET", `/models/${project.id}`))
      .filter((model) => model.name === name)
      .sort((a, b) => b.id - a.id);
    let id = null;
    for (const model of existing) {
      if (await downloads(backend, model.id)) {
        id = model.id;
        break;
      }
    }
    if (id === null) {
      const mib = (fs.statSync(file).size / 2 ** 20).toFixed(0);
      log(`uploading ${name} (${mib} MiB)`);
      const form = new FormData();
      form.append("file", await fs.openAsBlob(file), path.basename(file));
      const after = await api("POST", `/model/${project.id}/${encodeURIComponent(name)}`, form);
      id = Math.max(...after.filter((model) => model.name === name).map((model) => model.id));
      if (!(await downloads(backend, id))) throw new Error(`${name} was uploaded as ${id} but doesn't download`);
    }
    models.set(file, id);
  }
  return { project: project.id, models };
}

/** `docker compose up -d postgres backend` unless the backend answers already */
async function ensureBackend(root, backend, log) {
  if (await answers(backend)) return;
  log("starting postgres and backend: docker compose up -d postgres backend");
  execFileSync("docker", ["compose", "up", "-d", "postgres", "backend"], { cwd: root, stdio: "ignore" });
  for (let i = 0; i < 60; i++) {
    if (await answers(backend)) return;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`the backend at ${backend} doesn't answer after docker compose up`);
}

async function answers(backend) {
  try {
    await fetch(`${backend}/project`);
    return true;
  } catch {
    return false;
  }
}

async function logIn(backend, log) {
  const post = async (route, body) => {
    const response = await fetch(`${backend}${route}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    return response.json();
  };
  const login = await post("/login", { email: ACCOUNT.email, password: ACCOUNT.password });
  if (login.Ok) return login.Ok.token;
  log(`registering ${ACCOUNT.email}`);
  return unwrap(await post("/register", ACCOUNT), "POST /register").token;
}

async function downloads(backend, id) {
  try {
    const response = await fetch(`${backend}/assets/model/${id}.glb`, { method: "HEAD" });
    return response.ok;
  } catch {
    return false;
  }
}

/** The backend answers with a serialized Rust `Result`: `{"Ok": …}` or `{"Err": {message}}` */
function unwrap(result, what) {
  if (result && "Ok" in result) return result.Ok;
  throw new Error(`${what}: ${result?.Err?.message ?? JSON.stringify(result)}`);
}
