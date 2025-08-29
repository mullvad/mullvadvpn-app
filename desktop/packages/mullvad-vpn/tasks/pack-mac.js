const { build } = require('./build');
const { packMac } = require('./distribution');

async function buildAndPackage() {
  await build();
  await packMac();
}

buildAndPackage();
