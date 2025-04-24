const path = require('path');
const { removeRecursively, runCommand, setNodeEnvironment } = require('./utils');

const WORKSPACE_PROJECT_ROOT = path.resolve(__dirname, '..');
const BUILD_STANDALONE_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'build-standalone');

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
