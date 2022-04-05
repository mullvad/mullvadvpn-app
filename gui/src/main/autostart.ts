import { app } from 'electron';
import fs from 'fs';
import path from 'path';

import log from '../shared/logging';
import { getDesktopEntries } from './linux-desktop-entry';

const DESKTOP_FILE_NAME = 'mullvad-vpn.desktop';

export function getOpenAtLogin() {
  if (process.platform === 'linux') {
    try {
      const autostartDir = path.join(app.getPath('appData'), 'autostart');
      const autostartFilePath = path.join(autostartDir, DESKTOP_FILE_NAME);

      fs.accessSync(autostartFilePath);

      return true;
    } catch (e) {
      const error = e as Error;
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
      const desktopFilePath = await getDesktopEntryPath();
      const autostartDir = path.join(app.getPath('appData'), 'autostart');
      const autostartFilePath = path.join(autostartDir, DESKTOP_FILE_NAME);

      if (openAtLogin) {
        await createDirIfNecessary(autostartDir);
        await fs.promises.symlink(desktopFilePath, autostartFilePath);
      } else {
        await fs.promises.unlink(autostartFilePath);
      }
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to set auto-start: ${error.message}`);
    }
  } else {
    app.setLoginItemSettings({ openAtLogin });
  }
}

async function getDesktopEntryPath(): Promise<string> {
  const entries = await getDesktopEntries();
  const entry = entries.find((entry) => path.parse(entry).base === DESKTOP_FILE_NAME);
  if (entry) {
    return entry;
  } else {
    throw new Error(`Couldn't find ${DESKTOP_FILE_NAME}`);
  }
}

const createDirIfNecessary = async (directory: string) => {
  let stat;
  try {
    stat = await fs.promises.stat(directory);
  } catch {
    // Path doesn't exist, so it has to be created
    return fs.promises.mkdir(directory);
  }

  // Is there a file instead of a directory?
  if (!stat.isDirectory()) {
    // Try to remove existing file and replace it with a new directory
    try {
      await fs.promises.unlink(directory);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to remove path before creating a directory for it: ${error.message}`);
    }

    return fs.promises.mkdir(directory);
  }
};
