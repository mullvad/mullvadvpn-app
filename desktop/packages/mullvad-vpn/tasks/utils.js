const childProcess = require('child_process');
const fs = require('fs/promises');
const path = require('path');

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

exports.copyRecursively = copyRecursively;
exports.getCopyExtensionFilter = getCopyExtensionFilter;
exports.removeRecursively = removeRecursively;
exports.runCommand = runCommand;
exports.runNpmScript = runNpmScript;
exports.setNodeEnvironment = setNodeEnvironment;
