// This module loads the platform-specific build of the addon on
// the current system.

/* eslint-disable @typescript-eslint/no-require-imports */
module.exports = require('@neon-rs/load').proxy({
  platforms: {
    'win32-x64-msvc': () => require('../dist/win32-x64-msvc'),
    'win32-arm64-msvc': () => require('../dist/win32-arm64-msvc'),
  },
  debug: () => require('../debug/index.node'),
});
