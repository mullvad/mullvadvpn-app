// @flow
import path from 'path';

export function getSystemTemporaryDirectory() {
  switch (process.platform) {
    case 'win32': {
      const windowsPath = process.env.windir;
      if (windowsPath) {
        return path.join(windowsPath, 'Temp');
      } else {
        throw new Error('Missing windir in environment variables.');
      }
    }
    case 'darwin':
    case 'linux':
      return '/tmp';
    default:
      throw new Error(`Not implemented for ${process.platform}`);
  }
}
