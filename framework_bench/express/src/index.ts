// Minimal Express server for experiment A (framework without a database).
// Every endpoint mirrors framework_bench/axum/src/main.rs; see framework_bench/PLAN.md.
import express, { Request, Response } from "express"
import argon2 from "argon2"
import fs from "fs"
import http from "http"
import path from "path"

// Same pepper and parameters as backend-express/src/controllers/auth.ts and backend.
const PEPPER_SECRET = Buffer.from("mG7JFCuK1/wyoyyaQ1N9lQ")
// Fixed salt, so both servers must return the byte-identical PHC string.
const SALT = Buffer.from("framework-bench!")
const PASSWORD = "user@mail.com"
const ARGON2_HASH_OPTIONS = {
  secret: PEPPER_SECRET,
  salt: SALT,
  type: argon2.argon2id,
  memoryCost: 19000,
  timeCost: 2,
  parallelism: 1,
  hashLength: 32,
} as const

const LARGE_JSON_ITEMS = 1600
const DESCRIPTION =
  "Window frame on the north facade does not match the detail drawing; check the lintel height and the sill offset before ordering."
const STATUSES = ["TODO", "IN_PROGRESS", "DONE"]
const MAX_DELAY_MS = 10_000
const BODY_LIMIT = "16mb"

const port = Number(process.env.PORT || 8080)
const filePath = path.resolve(
  process.env.FILE_PATH ||
    path.join(__dirname, "../../../loadtest/src/loadtest.glb"),
)
if (!fs.statSync(filePath, { throwIfNoEntry: false })?.isFile()) {
  throw new Error(`FILE_PATH ${filePath} does not exist`)
}
// hyper never closes idle keep-alive connections; 0 disables Node's timeout to match.
const keepAliveTimeout = Number(process.env.KEEP_ALIVE_TIMEOUT_MS || 0)

// Field order matches the Rust struct so the serialized bytes are comparable.
const largeJsonItems = Array.from({ length: LARGE_JSON_ITEMS }, (_, i) => ({
  id: i,
  project_id: Math.floor(i / 50) + 1,
  title: `Tag ${i}`,
  description: DESCRIPTION,
  status: STATUSES[i % 3],
  created_by: "user@mail.com",
  position: { x: i * 0.25, y: i * 0.5 - 100, z: i * 0.125 },
}))

interface EchoItem {
  name: string
  value: number
}

interface EchoIn {
  id: number
  title: string
  items: EchoItem[]
}

// Same checks serde performs when deserializing EchoIn on the Axum side.
function isEchoIn(body: any): body is EchoIn {
  return (
    body !== null &&
    typeof body === "object" &&
    Number.isInteger(body.id) &&
    typeof body.title === "string" &&
    Array.isArray(body.items) &&
    body.items.every(
      (item: any) =>
        item !== null &&
        typeof item === "object" &&
        typeof item.name === "string" &&
        typeof item.value === "number",
    )
  )
}

const app = express()
// Axum computes neither, so turn them off for parity.
app.set("etag", false)
app.set("x-powered-by", false)

app.get("/plaintext", (_req: Request, res: Response) => {
  res.type("text/plain").send("Hello, World!")
})

app.get("/json", (_req: Request, res: Response) => {
  res.json({ message: "Hello, World!" })
})

app.post(
  "/echo-json",
  express.json({ limit: BODY_LIMIT }),
  (req: Request, res: Response) => {
    const input = req.body
    if (!isEchoIn(input)) {
      res.status(422).type("text/plain").send("invalid body")
      return
    }
    let sum = 0
    for (const item of input.items) sum += item.value
    const items = input.items
      .map((item) => ({ name: item.name, value: item.value }))
      .reverse()
    res.json({
      id: input.id,
      title: input.title.toUpperCase(),
      count: items.length,
      sum,
      items,
    })
  },
)

// Serialized on every request from shared data.
app.get("/json-large", (_req: Request, res: Response) => {
  res.json(largeJsonItems)
})

app.get("/cpu", async (_req: Request, res: Response) => {
  try {
    const hash = await argon2.hash(PASSWORD, ARGON2_HASH_OPTIONS)
    res.type("text/plain").send(hash)
  } catch {
    res.sendStatus(500)
  }
})

app.get("/delay", async (req: Request, res: Response) => {
  const raw = req.query.ms
  if (raw !== undefined && (typeof raw !== "string" || !/^\d+$/.test(raw))) {
    res.status(400).type("text/plain").send("invalid ms")
    return
  }
  const ms = Math.min(raw === undefined ? 20 : Number(raw), MAX_DELAY_MS)
  await new Promise((resolve) => setTimeout(resolve, ms))
  res.type("text/plain").send("ok")
})

app.get("/file", (_req: Request, res: Response) => {
  res.sendFile(filePath, { etag: false })
})

app.post(
  "/upload",
  express.raw({ type: () => true, limit: BODY_LIMIT }),
  (req: Request, res: Response) => {
    res.json({ bytes: Buffer.isBuffer(req.body) ? req.body.length : 0 })
  },
)

// Node enables TCP_NODELAY by default, axum::serve does not, so disable it to match.
const server = http.createServer({ noDelay: false }, app)
server.keepAliveTimeout = keepAliveTimeout
server.headersTimeout = 120_000

server.listen(port, () => {
  console.log(
    `express listening on :${port}, pid ${process.pid}, keepAliveTimeout ${keepAliveTimeout}, UV_THREADPOOL_SIZE ${process.env.UV_THREADPOOL_SIZE ?? "default"}, file: ${filePath}`,
  )
})

process.on("SIGTERM", () => {
  server.close(() => process.exit(0))
  server.closeAllConnections()
})
