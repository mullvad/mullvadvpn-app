import { PlaywrightTestConfig } from '@playwright/test';
const config: PlaywrightTestConfig = {
  testDir: './e2e',
  maxFailures: 2,
  expect: {
    toMatchSnapshot: {
      threshold: 0.1,
      maxDiffPixelRatio: 0.01,
    },
  },
  use: {
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
};

export default config;
