const { BUILD_STANDALONE_DIR } = require('./constants');
const { removeRecursively, runCommand, setNodeEnvironment } = require('./utils');

async function transpileBuildStandalone() {
  await runCommand('npx tsc -p tsconfig.standalone.json');
}

async function cleanBuildStandaloneDirectory() {
  await removeRecursively(BUILD_STANDALONE_DIR);
}

async function buildStandalone() {
  await cleanBuildStandaloneDirectory();
  setNodeEnvironment('production');
  await transpileBuildStandalone();
}

buildStandalone();
