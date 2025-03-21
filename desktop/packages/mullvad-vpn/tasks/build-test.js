const { build } = require('./build.js');
const { setNodeEnvironment } = require('./utils.js');

async function buildTest() {
  setNodeEnvironment('test');
  await build();
}

buildTest();
