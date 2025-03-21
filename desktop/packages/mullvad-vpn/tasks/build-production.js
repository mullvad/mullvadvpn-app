const { build } = require('./build');
const { setNodeEnvironment } = require('./utils');

async function buildProduction() {
  setNodeEnvironment('production');
  await build();
}

buildProduction();
