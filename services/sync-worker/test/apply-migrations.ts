/**
 * Runs inside the Workers runtime (workerd) as a Vitest setupFile. Applies
 * migrations/0001_init.sql (handed in as the TEST_MIGRATIONS JSON binding,
 * see vitest.config.ts) to the in-memory D1 database before any test runs.
 */
import { env } from "cloudflare:workers";
import { applyD1Migrations, type D1Migration } from "cloudflare:test";

interface TestEnv {
  DB: D1Database;
  TEST_MIGRATIONS: D1Migration[];
}

const testEnv = env as unknown as TestEnv;
await applyD1Migrations(testEnv.DB, testEnv.TEST_MIGRATIONS);
