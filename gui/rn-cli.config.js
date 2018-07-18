const path = require('path');

console.log('React-native CLI is running at: ', __dirname);

module.exports = {
  getProjectRoots() {
    return [
      path.resolve(__dirname, 'packages/mobile'),
      path.resolve(__dirname, 'packages/components/'),
      path.resolve(__dirname, 'packages/mobile/node_modules'),
      path.resolve(__dirname, 'node_modules'),
    ];
  }
}