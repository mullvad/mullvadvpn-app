import os from 'os';

export function isMacOs11OrNewer(): boolean {
  const [major] = parseVersion();
  return process.platform === 'darwin' && major >= 20;
}

export function isMacOs13OrNewer(): boolean {
  const [major] = parseVersion();
  return process.platform === 'darwin' && major >= 22;
}

export function isMacOs14p6OrNewer(): boolean {
  const [major, minor] = parseVersion();
  const darwin24 = major >= 24;
  const darwin236 = major == 23 && minor >= 6; // 23.6 is used by macOS 14.6
  return process.platform === 'darwin' && (darwin236 || darwin24);
}

// Windows 11 has the internal version 10.0.22000+.
export function isWindows11OrNewer(): boolean {
  const [major, minor, patch] = parseVersion();
  return (
    process.platform === 'win32' && (major > 10 || (major === 10 && (minor > 0 || patch >= 22000)))
  );
}

function parseVersion(): number[] {
  return os
    .release()
    .split('.')
    .map((value) => parseInt(value, 10));
}
