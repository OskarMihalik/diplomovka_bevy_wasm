// Experiment B client, Kysely variant (PLAN.md, section 6). The Rust client
// (`../rust/src/main.rs`) runs the same scenario through Cornucopia; keep the two in step.
//
//   node dist/index.js --rep <n> [--workers 10] [--warmup 100] [--iterations 1000] [--smoke]
//
// The query builders are the ones in backend-express/src/controllers (auth.ts, tags.ts,
// projects.ts) with two changes so both variants send the same SQL: the login lookup selects the
// 5 columns Cornucopia selects instead of selectAll(), and the inserts leave updated_at to its
// column default. Kysely takes a pool connection per query (a transaction holds one), which matches
// one pool.get() per step in the Rust client.

import { Kysely, PostgresDialect } from "kysely"
import { Pool } from "pg"
import { DB } from "./db/types"

const POOL_SIZE = 10

const args = { rep: 0, workers: 10, warmup: 100, iterations: 1000, smoke: false }
for (let i = 2; i < process.argv.length; i++) {
  const flag = process.argv[i]
  if (flag === "--smoke") {
    args.smoke = true
    continue
  }
  const key = flag.replace(/^--/, "") as "rep" | "workers" | "warmup" | "iterations"
  if (!(key in args)) throw new Error(`unknown argument ${flag}`)
  args[key] = Number(process.argv[++i])
}

const connectionString =
  process.env.DATABASE_URL ?? "postgresql://postgres:postgres@qbench-pg:5432/qbench"
const pool = new Pool({ connectionString, max: POOL_SIZE })
const db = new Kysely<DB>({ dialect: new PostgresDialect({ pool }) })

interface Worker {
  userId: number
  projectId: number
  email: string
}

interface Samples {
  scenario: number[]
  steps: number[][]
  errors: number
  rows: number
  checksum: number
}

const now = () => process.hrtime.bigint()
const since = (t: bigint) => Number(process.hrtime.bigint() - t)

/** One scenario: step latencies in ns, rows read or written, content checksum (same formula as Rust). */
async function scenario(w: Worker): Promise<[number[], number, number]> {
  const lat = [0, 0, 0, 0]
  let checksum = 0

  // 1 user_by_email (login lookup, auth.ts)
  let t = now()
  const user = await db
    .selectFrom("User")
    .select(["id", "email", "username", "password", "salt"])
    .where("email", "=", w.email)
    .executeTakeFirstOrThrow()
  lat[0] = since(t)
  checksum += user.id + user.email.length + user.username.length

  // 2 tags_by_project (getTagsForProject, tags.ts)
  t = now()
  const tags = await db
    .selectFrom("Tag as t")
    .leftJoin("Status as s", "s.id", "t.status_id")
    .innerJoin("User as u", "u.id", "t.created_by_id")
    .select([
      "t.id", "t.title", "t.project_id", "t.created_at", "t.position_x", "t.position_y", "t.position_z",
      "t.scale_x", "t.scale_y", "t.scale_z", "t.rotation_x", "t.rotation_y", "t.rotation_z", "t.created_by_id",
      "u.email as user_email", "u.username as user_username",
      "s.title as status_title", "s.id as status_id", "s.color_r as status_color_r", "s.color_g as status_color_g",
      "s.color_b as status_color_b", "s.project_id as status_project_id", "s.shape as status_shape",
    ])
    .where("t.project_id", "=", w.projectId)
    .orderBy("t.created_at", "desc")
    .limit(100)
    .offset(0)
    .execute()
  lat[1] = since(t)
  for (const tag of tags) {
    checksum += tag.id + (tag.status_id ?? 0) + tag.title.length + Math.round(tag.position_x * 4)
  }

  // 3 insert_tag (insertTag, tags.ts)
  t = now()
  const inserted = await db
    .insertInto("Tag")
    .values({
      title: "qbench tag",
      project_id: w.projectId,
      position_x: 1.0,
      position_y: 2.0,
      position_z: 3.0,
      created_by_id: w.userId,
    })
    .executeTakeFirst()
  lat[2] = since(t)

  // 4 project_tx: insertProject's transaction (projects.ts), then deleteProject as a separate request
  t = now()
  const projectId = await db.transaction().execute(async (trx) => {
    const project = await trx
      .insertInto("Project")
      .values({
        name: "qbench tx project",
        description: "created and deleted by qbench",
        created_by_id: w.userId,
      })
      .returning("id")
      .executeTakeFirstOrThrow()
    await trx
      .insertInto("ProjectUser")
      .values({ project_id: project.id, user_id: w.userId, is_admin: true })
      .execute()
    return project.id
  })
  const deleted = await db.deleteFrom("Project").where("id", "=", projectId).executeTakeFirst()
  lat[3] = since(t)

  // rows: 1 user + tags + 1 tag + (1 project id + 1 membership) + 1 deleted project
  const rows =
    1 + tags.length + Number(inserted.numInsertedOrUpdatedRows ?? 0n) + 2 + Number(deleted.numDeletedRows)
  return [lat, rows, checksum]
}

async function runWorker(w: Worker, iterations: number, s: Samples) {
  for (let i = 0; i < iterations; i++) {
    const t = now()
    try {
      const [lat, rows, checksum] = await scenario(w)
      s.scenario.push(since(t))
      lat.forEach((v, step) => s.steps[step].push(v))
      s.rows += rows
      s.checksum += checksum
    } catch (error) {
      s.errors++
      console.error(`worker ${w.userId}: ${error}`)
    }
  }
}

async function phase(workers: number, iterations: number): Promise<Samples> {
  const s: Samples = { scenario: [], steps: [[], [], [], []], errors: 0, rows: 0, checksum: 0 }
  const all = []
  for (let id = 1; id <= workers; id++) {
    all.push(runWorker({ userId: id, projectId: id, email: `qbench${id}@bench.local` }, iterations, s))
  }
  await Promise.all(all)
  return s
}

/** Nearest-rank percentile in ms of a sorted array of ns. */
function pct(sorted: number[], q: number): number {
  if (sorted.length === 0) return 0
  const rank = Math.min(Math.max(Math.ceil(q * sorted.length), 1), sorted.length)
  return sorted[rank - 1] / 1e6
}

async function main() {
  // open all connections before the warm-up, so connection setup is never measured
  const held = await Promise.all(Array.from({ length: POOL_SIZE }, () => pool.connect()))
  held.forEach((client) => client.release())

  if (args.warmup > 0) await phase(args.workers, args.warmup)
  const cpuStart = process.cpuUsage()
  const start = now()
  const s = await phase(args.workers, args.iterations)
  const durationS = since(start) / 1e9
  const cpu = process.cpuUsage(cpuStart)
  const cpuS = (cpu.user + cpu.system) / 1e6

  if (args.smoke) {
    console.log(`${s.checksum} ${s.rows}`)
  } else {
    const n = s.scenario.length
    const byValue = (a: number, b: number) => a - b
    s.scenario.sort(byValue)
    s.steps.forEach((step) => step.sort(byValue))
    const mean = s.scenario.reduce((a, b) => a + b, 0) / Math.max(n, 1) / 1e6
    const row: string[] = [
      "kysely", "node", "kysely+pg",
      String(args.rep), String(args.workers), String(POOL_SIZE), String(n),
      durationS.toFixed(3), (n / durationS).toFixed(1), String(s.errors), String(s.rows),
      mean.toFixed(4), pct(s.scenario, 0.5).toFixed(4), pct(s.scenario, 0.95).toFixed(4), pct(s.scenario, 0.99).toFixed(4),
    ]
    for (const step of s.steps) row.push(pct(step, 0.5).toFixed(4), pct(step, 0.99).toFixed(4))
    row.push(cpuS.toFixed(3), ((cpuS * 1000) / Math.max(n, 1)).toFixed(4))
    row.push((process.resourceUsage().maxRSS / 1024).toFixed(1))
    console.log(row.join(","))
  }
  await db.destroy()
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
