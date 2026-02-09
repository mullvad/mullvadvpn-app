const { build } = require('./build.cjs');
const { setNodeEnvironment } = require('./utils.cjs');

async function buildTest() {
  setNodeEnvironment('test');
  await build();
}

buildTest();
