import { getRawAsset } from 'node:sea';

import * as child_process from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

const { spawn } = child_process;

// This file is bundled into a standalone executable able to run e2e tests against an installed
// version of the app. This file is the entrypoint in the executable and extracts the required
// assets and performs the tests. More info in /desktop/packages/mullvad-vpn/README.md.

let tmpDir: string = path.join(os.tmpdir(), 'mullvad-standalone-tests');

async function main() {
  prepareTmpDir();
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
    'build-standalone',
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

    proc.stdout.on('data', (data: unknown) => console.log((data as string).toString()));
    proc.stderr.on('data', (data: unknown) => console.error((data as string).toString()));
    proc.on('close', (code, signal) => {
      if (signal) {
        console.log('Received signal:', signal);
      }

      resolve(code ?? (signal ? 1 : 0));
    });
  });
}

/// Ensure a clean temporary directory exists.
///
/// Removal of a previous directory can fail, e.g. because another process still holds files in
/// it. In that case, fall back to a uniquely named directory instead of failing with `EEXIST`.
function prepareTmpDir() {
  removeTmpDir();

  if (fs.existsSync(tmpDir)) {
    const fallbackDir = `${tmpDir}-${process.pid}`;
    console.warn(
      `Failed to remove old test directory '${tmpDir}', using '${fallbackDir}' instead`,
    );
    tmpDir = fallbackDir;
  }

  fs.mkdirSync(tmpDir);
}

function removeTmpDir() {
  if (fs.existsSync(tmpDir)) {
    try {
      // Retries are needed since busy files can cause the removal to fail transiently.
      fs.rmSync(tmpDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 1000 });
    } catch (e) {
      const error = e as Error;
      console.error('Failed to remove tmp dir:', error.message);
    }
  }
}

void main();
