import os from 'os';

export function isMacOs11OrNewer() {
  const [major] = parseVersion();
  return process.platform === 'darwin' && major >= 20;
}

// Windows 11 has the internal version 10.0.22000+.
export function isWindows11OrNewer() {
  const [major, minor, patch] = parseVersion();
  return (
    process.platform === 'win32' && (major > 10 || (major === 10 && (minor > 0 || patch >= 22000)))
  );
}

function parseVersion() {
  return os
    .release()
    .split('.')
    .map((value) => parseInt(value, 10));
}
