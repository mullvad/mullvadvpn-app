import { spawnSync, SpawnSyncReturns } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';

// This file is bundled into a standalone executable able to run e2e tests agains an installed
// version of the app. This file is the entrypoint in the executable and extracts the required
// assets and performs the tests. More info in /gui/README.md.

const tmpDir = path.join(os.tmpdir(), 'mullvad-standalone-tests');
const rootDir = path.join(__dirname, '..');
const nodeModulesDir = path.join(rootDir, 'node_modules');
const srcDir = path.join(rootDir, 'build', 'src');
const testDir = path.join(rootDir, 'build', 'test');

const nodeBin = process.argv[0];
const playwrightBin = path.join(tmpDir, 'node_modules', '@playwright', 'test', 'cli.js');

function main() {
  extract();

  // Tests need to be run sequentially since they interact with the same daemon instance.
  // Arguments are forwarded to playwright to make it possible to run specific tests.
  const args = [playwrightBin, 'test', '--workers', '1', ...process.argv.slice(2)];
  const result = spawnSync(nodeBin, args, { encoding: 'utf8', cwd: tmpDir });

  removeTmpDir();
  handleResult(result);
}

function extract() {
  // Remove old directory if already existing and create new clean one
  removeTmpDir();
  fs.mkdirSync(tmpDir);

  extractDirectory(srcDir);
  extractDirectory(testDir);
  extractDirectory(nodeModulesDir);
}

function extractDirectory(source: string) {
  copyRecursively(source, path.join(tmpDir, path.basename(source)));
}

function copyRecursively(source: string, target: string) {
  if (fs.statSync(source).isDirectory()) {
    fs.mkdirSync(target);
    fs.readdirSync(source, { encoding: 'utf8' }).forEach((item) =>
      copyRecursively(path.join(source, item), path.join(target, item)),
    );
  } else {
    fs.copyFileSync(source, target);
  }
}

function handleResult(result: SpawnSyncReturns<string>) {
  // Forward all output from playwright
  console.log(result.stdout);
  console.error(result.stderr);
  if (result.error) {
    console.error(result.error);
  }

  // Exit with the same exit code as playwright
  if (result.status === null) {
    process.exit(1);
  } else {
    process.exit(result.status);
  }
}

function removeTmpDir() {
  if (fs.existsSync(tmpDir)) {
    fs.rmSync(tmpDir, { recursive: true });
  }
}

main();
