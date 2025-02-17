import { getRawAsset } from 'node:sea';

import child_process, { spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';

// This file is bundled into a standalone executable able to run e2e tests against an installed
// version of the app. This file is the entrypoint in the executable and extracts the required
// assets and performs the tests. More info in /desktop/packages/mullvad-vpn/README.md.

const tmpDir = path.join(os.tmpdir(), 'mullvad-standalone-tests');

async function main() {
  extract();

  const code = await runTests();

  removeTmpDir();
  process.exit(code);
}

function getTarBin() {
  if (process.platform === 'win32') {
    if (process.env.windir) {
      return path.join(process.env.windir, 'System32', 'tar.exe');
    }
    return 'tar.exe';
  } else {
    return 'tar';
  }
}

function extract() {
  // Remove old directory if already existing and create new clean one
  removeTmpDir();
  fs.mkdirSync(tmpDir);

  // Copy assets archive to temp dir
  const tarAssets = getRawAsset('assets.tar.gz') as ArrayBuffer;
  fs.writeFileSync(path.join(tmpDir, 'assets.tar.gz'), Buffer.from(tarAssets));

  // Untar assets
  const args = ['-xzf', path.join(tmpDir, 'assets.tar.gz')];
  child_process.spawnSync(getTarBin(), args, { cwd: tmpDir });
}

function getNodeBin() {
  if (process.platform === 'win32') {
    return path.join(tmpDir, 'node.exe');
  } else {
    return path.join(tmpDir, 'node');
  }
}

function createSealessNode() {
  const nodeBin = getNodeBin();

  fs.copyFileSync(process.argv[0], nodeBin);

  if (process.platform === 'darwin') {
    child_process.spawnSync('/usr/bin/codesign', ['--remove-signature', nodeBin]);
  }

  // Find and disable SEA fuse in node binary
  const fuseString = 'NODE_SEA_FUSE_' + 'fce680ab2cc467b6e072b8b5df1996b2:';

  const buf = fs.readFileSync(nodeBin);
  const fuseIndex = buf.indexOf(fuseString);

  if (fuseIndex !== -1) {
    const stateIndex = fuseIndex + fuseString.length;
    if (stateIndex < buf.length && buf[stateIndex] === '1'.charCodeAt(0)) {
      // If we set the state of the fuse to 0, it will not execute our payload
      buf[stateIndex] = '0'.charCodeAt(0);
      fs.writeFileSync(nodeBin, buf);
      fs.chmodSync(nodeBin, 0o554);
    }
  }

  if (process.platform === 'darwin') {
    child_process.spawnSync('/usr/bin/codesign', ['--sign', '-', nodeBin]);
  }

  return nodeBin;
}

function runTests(): Promise<number> {
  const nodeBin = createSealessNode();
  const playwrightBin = path.join(tmpDir, 'node_modules', '@playwright', 'test', 'cli.js');
  const configPath = path.join(
    tmpDir,
    'standalone',
    'test',
    'e2e',
    'installed',
    'playwright.config.js',
  );

  return new Promise((resolve) => {
    // Tests need to be run sequentially since they interact with the same daemon instance.
    // Arguments are forwarded to playwright to make it possible to run specific tests.
    const args = [playwrightBin, 'test', '-x', '-c', configPath, ...process.argv.slice(2)];
    const proc = spawn(nodeBin, args, { cwd: tmpDir });

    proc.stdout.on('data', (data) => console.log(data.toString()));
    proc.stderr.on('data', (data) => console.error(data.toString()));
    proc.on('close', (code, signal) => {
      if (signal) {
        console.log('Received signal:', signal);
      }

      resolve(code ?? (signal ? 1 : 0));
    });
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
