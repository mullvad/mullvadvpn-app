const { build } = require('./build');
const { packWin } = require('./distribution');

async function buildAndPackage() {
  await build();
  await packWin();
}

buildAndPackage();
