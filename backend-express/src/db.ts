import { Pool } from "pg"
import { Kysely, PostgresDialect } from "kysely"
import { DB } from "./db/types"
import dotenv from "dotenv"
import path from "path"

dotenv.config({ path: path.join(__dirname, "../../.env") })

const dbHost =
  process.env.DB_POSTGRES_HOST || process.env.DB_HOST || "localhost"
const dbPort = parseInt(
  process.env.DB_POSTGRES_PORT || process.env.DB_PORT || "5438",
  10,
)
const dbUser = process.env.DB_POSTGRES_USER || "postgres"
const dbPassword = process.env.DB_POSTGRES_PASSWORD || "postgres"
const dbName = process.env.DB_POSTGRES_DB || "bevy"

let connectionString = process.env.DATABASE_URL
if (!connectionString) {
  connectionString = `postgresql://${dbUser}:${dbPassword}@${dbHost}:${dbPort}/${dbName}`
}

// DB_POOL_SIZE is per process and defaults to node-postgres' default of 10 connections; with
// WORKERS > 1 (cluster.ts) the backend holds WORKERS * DB_POOL_SIZE connections
const poolSize = Number(process.env.DB_POOL_SIZE || 10)
if (!Number.isInteger(poolSize) || poolSize < 1) {
  throw new Error(`invalid DB_POOL_SIZE=${process.env.DB_POOL_SIZE}`)
}

const dialect = new PostgresDialect({
  pool: new Pool({
    connectionString,
    max: poolSize,
  }),
})

export const db = new Kysely<DB>({
  dialect,
})
