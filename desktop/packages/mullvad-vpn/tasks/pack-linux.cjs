const { build } = require('./build.cjs');
const { packLinux } = require('./distribution.cjs');

async function buildAndPackage() {
  await build();
  await packLinux();
}

buildAndPackage();
