const { BUILD_STANDALONE_DIR } = require('./constants.cjs');
const { removeRecursively, runCommand, setNodeEnvironment } = require('./utils.cjs');

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
