import { app } from 'electron';
import * as fs from 'fs';
import * as path from 'path';
import { promisify } from 'util';
import log from '../shared/logging';

const DESKTOP_FILE_NAME = 'mullvad-vpn.desktop';

const mkdirAsync = promisify(fs.mkdir);
const statAsync = promisify(fs.stat);
const symlinkAsync = promisify(fs.symlink);
const unlinkAsync = promisify(fs.unlink);

export function getOpenAtLogin() {
  if (process.platform === 'linux') {
    try {
      const autostartDir = path.join(app.getPath('appData'), 'autostart');
      const autostartFilePath = path.join(autostartDir, DESKTOP_FILE_NAME);

      fs.accessSync(autostartFilePath);

      return true;
    } catch (error) {
      log.error(`Failed to check autostart file: ${error.message}`);
      return false;
    }
  } else {
    return app.getLoginItemSettings().openAtLogin;
  }
}

export async function setOpenAtLogin(openAtLogin: boolean) {
  if (process.platform === 'linux') {
    try {
      const desktopFilePath = path.join('/usr/share/applications', DESKTOP_FILE_NAME);
      const autostartDir = path.join(app.getPath('appData'), 'autostart');
      const autostartFilePath = path.join(autostartDir, DESKTOP_FILE_NAME);

      if (openAtLogin) {
        await createDirIfNecessary(autostartDir);
        await symlinkAsync(desktopFilePath, autostartFilePath);
      } else {
        await unlinkAsync(autostartFilePath);
      }
    } catch (error) {
      log.error(`Failed to set auto-start: ${error.message}`);
    }
  } else {
    app.setLoginItemSettings({ openAtLogin });
  }
}

const createDirIfNecessary = async (directory: string) => {
  let stat;
  try {
    stat = await statAsync(directory);
  } catch (error) {
    // Path doesn't exist, so it has to be created
    return mkdirAsync(directory);
  }

  // Is there a file instead of a directory?
  if (!stat.isDirectory()) {
    // Try to remove existing file and replace it with a new directory
    try {
      await unlinkAsync(directory);
    } catch (error) {
      log.error(`Failed to remove path before creating a directory for it: ${error.message}`);
    }

    return mkdirAsync(directory);
  }
};
