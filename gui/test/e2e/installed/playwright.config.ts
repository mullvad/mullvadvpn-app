import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: process.cwd(),
  timeout: 60_000,
  workers: 1,
  reportSlowTests: null,
  maxFailures: 1,
  expect: {
    timeout: 30_000,
  },
});
