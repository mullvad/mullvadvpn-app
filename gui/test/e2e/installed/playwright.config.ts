import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: process.cwd(),
  timeout: 60_000,
  workers: 1,
  expect: {
    timeout: 10_000,
  },
});
