// Runs index.ts as a node:cluster with WORKERS processes, so that Express can use as many CPUs as
// the Tokio runtime of the Axum backend. With WORKERS=1 (the default) it runs in this process.
import cluster from "node:cluster"

const workers = Number(process.env.WORKERS || 1)
if (!Number.isInteger(workers) || workers < 1) {
  console.error(`invalid WORKERS=${process.env.WORKERS}`)
  process.exit(2)
}

if (workers === 1 || !cluster.isPrimary) {
  require("./index")
} else {
  // Round-robin is Node's default on Linux; set it explicitly so the setup is documented.
  cluster.schedulingPolicy = cluster.SCHED_RR
  console.log(`cluster primary ${process.pid}, forking ${workers} workers`)
  for (let i = 0; i < workers; i++) cluster.fork()

  let stopping = false
  const stop = () => {
    stopping = true
    for (const worker of Object.values(cluster.workers ?? {})) worker?.kill("SIGTERM")
  }
  cluster.on("exit", (worker, code, signal) => {
    if (stopping) return
    // A dead worker would silently reduce capacity mid-run, so fail loudly instead.
    console.error(`worker ${worker.process.pid} exited (${signal || code}), stopping cluster`)
    process.exitCode = 1
    stop()
  })
  process.on("SIGTERM", stop)
}
