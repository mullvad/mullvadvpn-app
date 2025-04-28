const { build } = require('./build');
const { packLinux } = require('./distribution');

async function buildAndPackage() {
  await build();
  await packLinux();
}

buildAndPackage();
