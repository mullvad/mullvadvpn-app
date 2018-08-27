// This is a companion script that reverts the effect of preinstall script in 
// `\gui\packages\yarn-fixes`.
//
// The symlink to `\gui\node_modules\node_modules` that fixes the bug, described in 
// https://github.com/yarnpkg/yarn/issues/4564, must be removed after node modules installation, 
// because circular symlinks cause scripts like electron-builder to crash.

const path = require('path');
const fs = require('fs');

if (process.platform !== 'win32') {
  return;
}

const symlinkPath = path.join(__dirname, 'node_modules/node_modules');

try {
  console.log('Removing a symlink to node_modules/node_modules');
  fs.unlinkSync(symlinkPath);
} catch (error) {
  if (error.code !== 'ENOENT') {
    throw error;
  }
}
