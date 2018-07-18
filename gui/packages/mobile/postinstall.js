//
// React-native CLI doesn't function properly in workspace configuration.
//
// Symlinking `/gui/node_modules/react-native` to `/gui/packages/mobile/node_modules/react-native`
// solves this. See rn-cli.config.js for project roots override.
//

const path = require('path');
const fs = require('fs');

const sourcePath = path.resolve(path.join(__dirname, '../../node_modules/react-native'));
const symlinkPath = path.join(__dirname, 'node_modules/react-native');

try {
  console.log('Removing a symlink to react-native');
  fs.unlinkSync(symlinkPath);
} catch (error) {
  if (error.code !== 'ENOENT') {
    throw error;
  }
}

try {
  console.log('Adding a symlink to react-native');
  fs.symlinkSync(sourcePath, symlinkPath);
  console.log('Done');
} catch (error) {
  console.error('Cannot symlink react-native: ' + error.message);
}
