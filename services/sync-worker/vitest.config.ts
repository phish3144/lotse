import { defineConfig } from "vitest/config";
import { cloudflareTest, readD1Migrations } from "@cloudflare/vitest-pool-workers";

export default defineConfig({
  plugins: [
    cloudflareTest(async () => ({
      wrangler: { configPath: "./wrangler.toml" },
      miniflare: {
        // Migrations are read here (Node.js side, relative to this file's
        // directory since vitest runs with that as its cwd) and handed to
        // the worker as a plain JSON binding; test/apply-migrations.ts
        // applies them inside the workerd runtime before each test file.
        bindings: {
          TEST_MIGRATIONS: await readD1Migrations("./migrations"),
        },
      },
    })),
  ],
  test: {
    setupFiles: ["./test/apply-migrations.ts"],
  },
});
