// One origin for the web benchmarks: the two dists, the model of the running job and the
// `/report` endpoint the pages post their results to (see report.rs and report.js).

import fs from "node:fs";
import http from "node:http";
import path from "node:path";

const TYPES = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".css": "text/css",
  ".json": "application/json",
  ".glb": "model/gltf-binary",
  ".gltf": "model/gltf+json",
};

// cross-origin isolated pages: Firefox rounds performance.now() to 20 µs instead of 1 ms
const ISOLATION = {
  "cross-origin-opener-policy": "same-origin",
  "cross-origin-embedder-policy": "require-corp",
};

/**
 * `mounts` maps url prefixes like `/bevy/` to folders. `server.model` is the file served at
 * `/models/<its name>`, `server.onReport` gets every message posted to `/report`.
 */
export async function startServer(mounts, port) {
  const state = { model: null, onReport: () => {} };

  const server = http.createServer((request, response) => {
    const url = new URL(request.url, "http://localhost");
    if (request.method === "POST" && url.pathname === "/report") {
      let body = "";
      request.on("data", (chunk) => (body += chunk));
      request.on("end", () => {
        response.writeHead(204).end();
        try {
          state.onReport(JSON.parse(body));
        } catch (error) {
          state.onReport({ error: `bad report: ${error.message}` });
        }
      });
      return;
    }
    for (const [key, value] of Object.entries(ISOLATION)) response.setHeader(key, value);
    // empty page for the check before the batch (jobs.js, checkFirefox)
    if (url.pathname === "/check") {
      response.writeHead(200, { "content-type": "text/html" }).end("<!doctype html><title>check</title>");
      return;
    }

    let file;
    if (url.pathname.startsWith("/models/")) {
      const name = decodeURIComponent(url.pathname.slice("/models/".length));
      if (state.model && name === path.basename(state.model)) file = state.model;
    } else {
      const prefix = Object.keys(mounts).find((p) => url.pathname.startsWith(p));
      if (prefix) {
        const base = mounts[prefix];
        file = path.resolve(base, decodeURIComponent(url.pathname.slice(prefix.length)) || "index.html");
        if (file !== base && !file.startsWith(base + path.sep)) file = undefined;
      }
    }
    if (!file) return response.writeHead(404).end();
    fs.stat(file, (error, stat) => {
      if (!error && stat.isDirectory()) {
        file = path.join(file, "index.html");
        stat = fs.existsSync(file) ? fs.statSync(file) : null;
      }
      if (error || !stat) return response.writeHead(404).end();
      response.writeHead(200, {
        "content-type": TYPES[path.extname(file)] ?? "application/octet-stream",
        "content-length": stat.size,
        // every job starts with an empty browser cache anyway
        "cache-control": "no-store",
      });
      fs.createReadStream(file).pipe(response);
    });
  });

  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", resolve);
  });
  return {
    origin: `http://127.0.0.1:${port}`,
    set model(file) {
      state.model = file;
    },
    set onReport(handler) {
      state.onReport = handler;
    },
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}
