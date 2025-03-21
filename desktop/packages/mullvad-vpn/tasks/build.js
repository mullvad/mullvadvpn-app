const path = require('path');
const {
  copyRecursively,
  getCopyExtensionFilter,
  removeRecursively,
  runCommand,
} = require('./utils');

const WORKSPACE_PROJECT_ROOT = path.resolve(__dirname, '..');
const BUILD_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'build');

const GEO_DIR = path.resolve('../../../dist-assets/geo');
const ICONS_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'assets/icons');
const IMAGES_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'assets/images');
const LOCALES_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'locales');

async function copyToBuildFolder(sourcePath, folderName, extension) {
  const destinationPath = path.join(BUILD_DIR, folderName);
  const copyExtensionFilter = extension ? getCopyExtensionFilter(extension) : undefined;

  await copyRecursively(sourcePath, destinationPath, copyExtensionFilter);
}

async function copyAssetsToBuildDirectory() {
  await Promise.all([
    copyToBuildFolder(IMAGES_DIR, 'assets/images'),
    copyToBuildFolder(ICONS_DIR, 'assets/icons'),
    copyToBuildFolder(GEO_DIR, 'assets/geo', '.gl'),
    copyToBuildFolder(LOCALES_DIR, 'locales', '.po'),
  ]);
}

async function build() {
  await removeRecursively(BUILD_DIR);
  await runCommand('npm run build:vite');
  await copyAssetsToBuildDirectory();
}

exports.build = build;
