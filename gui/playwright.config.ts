import { PlaywrightTestConfig } from '@playwright/test';
import path from 'path';

const config: PlaywrightTestConfig = {
  testDir: './test/e2e',
  maxFailures: 2,
  timeout: 60000,
  snapshotDir: path.join(__dirname, '..', 'ci', 'screenshots', 'desktop'),
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
