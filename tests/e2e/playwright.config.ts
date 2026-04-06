import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./specs",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "on-first-retry",
  },
  webServer: {
    command:
      "bash -lc 'cd fixture-app && npm install --no-fund --no-audit && cargo run --manifest-path ../../../compiler/Cargo.toml -- build --config tinyfx.toml && npx serve dist -l 4173 --no-request-logging'",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: false,
    stdout: "ignore",
    stderr: "pipe",
  },
});
