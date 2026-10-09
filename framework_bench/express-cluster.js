// Runs an Express entry point as a node:cluster with WORKERS processes (experiments A and C, 4c).
// With WORKERS=1 (the default) the entry point runs directly in this process, without a primary.
//
//   WORKERS=4 node framework_bench/express-cluster.js framework_bench/express/dist/index.js
const cluster = require("node:cluster")
const path = require("node:path")

const entry = process.argv[2]
if (!entry) {
  console.error("usage: node express-cluster.js <entry.js>")
  process.exit(2)
}
const workers = Number(process.env.WORKERS || 1)
if (!Number.isInteger(workers) || workers < 1) {
  console.error(`invalid WORKERS=${process.env.WORKERS}`)
  process.exit(2)
}

if (workers === 1 || !cluster.isPrimary) {
  require(path.resolve(entry))
} else {
  // Round-robin is Node's default on Linux; set it explicitly so the setup is documented.
  cluster.schedulingPolicy = cluster.SCHED_RR
  console.log(`cluster primary ${process.pid}, forking ${workers} workers`)
  for (let i = 0; i < workers; i++) cluster.fork()

  let stopping = false
  cluster.on("exit", (worker, code, signal) => {
    if (stopping) return
    // A dead worker would silently reduce capacity mid-run, so fail loudly instead.
    console.error(`worker ${worker.process.pid} exited (${signal || code}), stopping cluster`)
    stopping = true
    for (const w of Object.values(cluster.workers)) w.kill("SIGTERM")
    process.exitCode = 1
  })
  process.on("SIGTERM", () => {
    stopping = true
    for (const w of Object.values(cluster.workers)) w.kill("SIGTERM")
  })
}
