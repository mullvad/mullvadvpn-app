import fs from 'fs';

export function fileExistsSync(path: string): boolean {
  try {
    fs.accessSync(path, fs.constants.F_OK);
    return true;
  } catch (e) {
    return false;
  }
}
