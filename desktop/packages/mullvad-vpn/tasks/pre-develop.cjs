const { copyAssetsToBuildDirectory, setNodeEnvironment } = require('./utils.js');

async function preDevelop() {
  setNodeEnvironment('development');
  await copyAssetsToBuildDirectory();
}

preDevelop();
