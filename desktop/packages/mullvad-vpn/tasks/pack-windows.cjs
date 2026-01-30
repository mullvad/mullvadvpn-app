const { build } = require('./build.cjs');
const { packWin } = require('./distribution.cjs');

async function buildAndPackage() {
  await build();
  await packWin();
}

buildAndPackage();
