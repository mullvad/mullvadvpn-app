// @flow

import fs from 'fs';

export function canTrustRpcAddressFile(path: string): boolean {
  const platform = process.platform;
  switch(platform) {
  case 'win32':
    return isOwnedByLocalSystem(path);
  case 'darwin':
  case 'linux':
    return isOwnedAndOnlyWritableByRoot(path);
  default:
    throw new Error(`Unknown platform: ${platform}`);
  }
}

function isOwnedAndOnlyWritableByRoot(path: string): boolean {
  const stat = fs.statSync(path);
  const isOwnedByRoot = stat.uid === 0;
  const isOnlyWritableByOwner = (stat.mode & parseInt('022', 8)) === 0;

  return isOwnedByRoot && isOnlyWritableByOwner;
}

function isOwnedByLocalSystem(path: string): boolean {
  const winsec = require('windows-security');
  const ownerSid = winsec.getFileOwnerSid(path, null);
  const isWellKnownSid = winsec.isWellKnownSid(ownerSid, winsec.WellKnownSid.LocalSystemSid);

  return isWellKnownSid;
}