const childProcess = require('child_process');
const fs = require('fs/promises');
const path = require('path');
const { BUILD_DIR, GEO_DIR, LOCALES_DIR, IMAGES_DIR, ICONS_DIR } = require('./constants');

async function copyAssetsToBuildDirectory() {
  await Promise.all([
    copyToBuildFolder(IMAGES_DIR, 'assets/images'),
    copyToBuildFolder(ICONS_DIR, 'assets/icons'),
    copyToBuildFolder(GEO_DIR, 'assets/geo', '.gl'),
    copyToBuildFolder(LOCALES_DIR, 'locales', '.po'),
  ]);
}

async function copyRecursively(sourcePath, destinationPath, filter) {
  await fs.mkdir(destinationPath, {
    recursive: true,
  });

  const sourceFiles = await fs.readdir(sourcePath, {
    recursive: true,
  });

  for (const sourceFile of sourceFiles) {
    const sourceFileAbsolute = path.join(sourcePath, sourceFile);
    const destinationPathAbsolute = path.join(destinationPath, sourceFile);

    await fs.cp(sourceFileAbsolute, destinationPathAbsolute, {
      filter,
      recursive: true,
    });
  }
}

async function copyToBuildFolder(sourcePath, folderName, extension) {
  const destinationPath = path.join(BUILD_DIR, folderName);
  const copyExtensionFilter = extension ? getCopyExtensionFilter(extension) : undefined;

  await copyRecursively(sourcePath, destinationPath, copyExtensionFilter);
}

function getCopyExtensionFilter(extension) {
  const copyExtensionFilter = (filePath) => {
    const fileExtension = path.extname(filePath);

    return fileExtension === extension;
  };

  return copyExtensionFilter;
}

async function removeRecursively(path) {
  await fs.rm(path, {
    recursive: true,
    force: true,
  });
}

async function runNpmScript(scriptName) {
  const command = `npm run ${scriptName}`;

  try {
    await runCommand(command);
  } catch (errors) {
    if (Array.isArray(errors)) {
      // Remove first error as it will always
      // bubble up and be printed to the console.
      errors.slice(1).forEach((error) => {
        console.error(new Error(error));
      });
    }
    process.exit(1);
  }
}

async function runCommand(command) {
  return new Promise((resolve, reject) => {
    childProcess.exec(command, (error, stdout, stderr) => {
      if (error) {
        return reject([error, stdout, stderr]);
      }

      return resolve([stdout, stderr]);
    });
  });
}

function setNodeEnvironment(environment) {
  process.env.NODE_ENV = environment;
}

exports.copyAssetsToBuildDirectory = copyAssetsToBuildDirectory;
exports.copyRecursively = copyRecursively;
exports.copyToBuildFolder = copyToBuildFolder;
exports.getCopyExtensionFilter = getCopyExtensionFilter;
exports.removeRecursively = removeRecursively;
exports.runCommand = runCommand;
exports.runNpmScript = runNpmScript;
exports.setNodeEnvironment = setNodeEnvironment;
