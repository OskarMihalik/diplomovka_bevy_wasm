// Posts results to rendering_runner when the url has `?report=<url>`, one message after another
// so `done` arrives after the last row. The same messages as bevy_rendering_benchmark:
// `{file, header, row}` for every CSV row, `{done: true}` at the end, `{error}`.

const url = new URLSearchParams(location.search).get("report");
let queue = Promise.resolve();

export function report(message) {
  if (!url) return;
  const body = JSON.stringify(message);
  queue = queue
    .then(() => fetch(url, { method: "POST", headers: { "content-type": "application/json" }, body }))
    .catch((error) => console.error("report failed:", error));
}

if (url) {
  addEventListener("error", (event) => report({ error: String(event.error ?? event.message) }));
  addEventListener("unhandledrejection", (event) => report({ error: String(event.reason) }));
}
