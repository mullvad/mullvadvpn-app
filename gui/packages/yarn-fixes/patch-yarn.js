// Yarn 1.9.4 has a path lookup bug on Windows, when it looks for the binaries referenced in
// scripts under '\gui\node_modules\node_modules' instead of '\gui\node_modules'.
// This patch adds a junction between those two to keep that house of cards from falling apart.
// GitHub issue: https://github.com/yarnpkg/yarn/issues/4564

const path = require('path');
const fs = require('fs');

if (process.platform !== 'win32') {
  return;
}

const sourcePath = path.resolve(path.join(__dirname, '../../node_modules'));
const symlinkPath = path.join(__dirname, '../../node_modules/node_modules');

try {
  console.log('Removing a symlink to node_modules/node_modules');
  fs.unlinkSync(symlinkPath);
} catch (error) {
  if (error.code !== 'ENOENT') {
    throw error;
  }
}

try {
  console.log('Applying yarn workspaces patch for node_modules/node_modules');
  fs.symlinkSync(sourcePath, symlinkPath, 'junction');
  console.log('Done');
} catch (error) {
  console.error('Cannot symlink node_modules/node_modules: ' + error.message);
}
