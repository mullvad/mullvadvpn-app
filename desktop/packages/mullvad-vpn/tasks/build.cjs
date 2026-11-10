const { copyAssetsToBuildDirectory, removeRecursively, runNpmScript } = require('./utils.cjs');
const { BUILD_DIR } = require('./constants.cjs');

async function build() {
  const args = process.argv.filter((arg) => arg.startsWith('--'));
  // Need three double hypens to pass through all subprocess down to `vite` correctly
  const buildArgs = ['--', '--', '--', ...args];

  await removeRecursively(BUILD_DIR);
  await runNpmScript('type-check');
  await runNpmScript('build:vite', buildArgs);
  await copyAssetsToBuildDirectory();
}

exports.build = build;
