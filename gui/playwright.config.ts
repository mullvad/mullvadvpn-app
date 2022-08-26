import { PlaywrightTestConfig } from '@playwright/test';
const config: PlaywrightTestConfig = {
  testDir: './e2e',
  maxFailures: 2,
  use: {
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
};

export default config;
