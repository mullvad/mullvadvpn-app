// This module loads the platform-specific build of the addon on
// the current system.

/* eslint-disable @typescript-eslint/no-require-imports */
module.exports = require('@neon-rs/load').proxy({
  platforms: {
    'darwin-x64': () => require('../dist/darwin-x64'),
    'darwin-arm64': () => require('../dist/darwin-arm64'),
  },
  debug: () => require('../target/debug/index.node'),
});
