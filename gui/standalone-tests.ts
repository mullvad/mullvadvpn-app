import { spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';

// This file is bundled into a standalone executable able to run e2e tests against an installed
// version of the app. This file is the entrypoint in the executable and extracts the required
// assets and performs the tests. More info in /gui/README.md.

const tmpDir = path.join(os.tmpdir(), 'mullvad-standalone-tests');

async function main() {
  extract();
  const code = await runTests();

  removeTmpDir();
  process.exit(code);
}

function extract() {
  const rootDir = path.join(__dirname, '..');
  const nodeModulesDir = path.join(rootDir, 'node_modules');
  const srcDir = path.join(rootDir, 'build', 'src');
  const testDir = path.join(rootDir, 'build', 'test');

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

function runTests(): Promise<number> {
  const nodeBin = process.argv[0];
  const playwrightBin = path.join(tmpDir, 'node_modules', '@playwright', 'test', 'cli.js');
  const configPath = path.join(tmpDir, 'test', 'e2e', 'installed', 'playwright.config.js');

  return new Promise((resolve) => {
    // Tests need to be run sequentially since they interact with the same daemon instance.
    // Arguments are forwarded to playwright to make it possible to run specific tests.
    const args = [playwrightBin, 'test', '-c', configPath, ...process.argv.slice(2)];
    const proc = spawn(nodeBin, args, { cwd: tmpDir });

    proc.stdout.on('data', (data) => console.log(data.toString()));
    proc.stderr.on('data', (data) => console.error(data.toString()));
    proc.on('close', resolve);
  });
}

function removeTmpDir() {
  if (fs.existsSync(tmpDir)) {
    try {
      fs.rmSync(tmpDir, { recursive: true });
    } catch (e) {
      const error = e as Error;
      console.error('Failed to remove tmp dir:', error.message);
    }
  }
}

void main();
