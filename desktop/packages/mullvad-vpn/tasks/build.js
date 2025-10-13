const { copyAssetsToBuildDirectory, removeRecursively, runNpmScript } = require('./utils');
const { BUILD_DIR } = require('./constants');

async function build() {
  await removeRecursively(BUILD_DIR);
  await runNpmScript('type-check');
  await runNpmScript('build:vite');
  await copyAssetsToBuildDirectory();
}

exports.build = build;
