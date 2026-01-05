const { copyAssetsToBuildDirectory, setNodeEnvironment } = require('./utils.cjs');

async function preDevelop() {
  setNodeEnvironment('development');
  await copyAssetsToBuildDirectory();
}

preDevelop();
