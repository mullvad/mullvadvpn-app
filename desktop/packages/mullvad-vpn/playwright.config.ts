import { PlaywrightTestConfig } from '@playwright/test';
const config: PlaywrightTestConfig = {
  testDir: './test/e2e',
  maxFailures: 1,
};

export default config;
