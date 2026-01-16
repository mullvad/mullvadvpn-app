const { build } = require('./build.cjs');
const { packMac } = require('./distribution.cjs');

async function buildAndPackage() {
  await build();
  await packMac();
}

buildAndPackage();
