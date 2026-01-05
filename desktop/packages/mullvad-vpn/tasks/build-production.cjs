const { build } = require('./build.cjs');
const { setNodeEnvironment } = require('./utils.cjs');

async function buildProduction() {
  setNodeEnvironment('production');
  await build();
}

buildProduction();
