import { Pool } from 'pg';
import { Kysely, PostgresDialect } from 'kysely';
import { DB } from './db/types';
import dotenv from 'dotenv';
import path from 'path';

dotenv.config({ path: path.join(__dirname, '../../.env') });

const dbHost = process.env.DB_HOST || 'localhost';
const dbPort = parseInt(process.env.DB_PORT || '5438', 10);
const dbUser = process.env.DB_POSTGRES_USER || 'postgres';
const dbPassword = process.env.DB_POSTGRES_PASSWORD || 'postgres';
const dbName = process.env.DB_POSTGRES_DB || 'bevy';

let connectionString = process.env.DATABASE_URL;
if (!connectionString) {
  connectionString = `postgresql://${dbUser}:${dbPassword}@${dbHost}:${dbPort}/${dbName}`;
}

const dialect = new PostgresDialect({
  pool: new Pool({
    connectionString,
  })
});

export const db = new Kysely<DB>({
  dialect,
});
