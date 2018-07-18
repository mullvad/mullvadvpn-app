// @flow

import fs from 'fs';
import path from 'path';
import { execFile } from 'child_process';
import { promisify } from 'util';
import { remote } from 'electron';
import log from 'electron-log';

const DESKTOP_FILE_NAME = 'mullvad-vpn.desktop';

const execFileAsync = promisify(execFile);
const mkdirAsync = promisify(fs.mkdir);
const statAsync = promisify(fs.stat);
const symlinkAsync = promisify(fs.symlink);
const unlinkAsync = promisify(fs.unlink);

export function getOpenAtLogin() {
  return remote.app.getLoginItemSettings().openAtLogin;
}

export async function setOpenAtLogin(openAtLogin: boolean) {
  // setLoginItemSettings is broken on macOS and cannot delete login items.
  // Issue: https://github.com/electron/electron/issues/10880
  if (process.platform === 'darwin') {
    if (openAtLogin === false) {
      // process.execPath in renderer process points to the sub-bundle of Electron Helper.
      // This regular expression extracts the path to the app bundle, which is the first occurrence of
      // file with .app extension.
      const matches = process.execPath.match(/([a-z0-9 ]+)\.app/i);
      if (matches && matches.length > 1) {
        const bundleName = matches[1];
        const appleScript = `on run argv
          set itemName to item 1 of argv
          tell application "System Events" to delete login item itemName
        end run`;
        await execFileAsync('osascript', ['-e', appleScript, bundleName]);
      } else {
        log.error(`Cannot extract the app bundle name from ${process.execPath}`);
      }
    } else {
      remote.app.setLoginItemSettings({ openAtLogin });
    }
  } else if (process.platform === 'linux') {
    try {
      const desktopFilePath = path.join('/usr/share/applications', DESKTOP_FILE_NAME);
      const autostartDir = path.join(remote.app.getPath('appData'), 'autostart');
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
    remote.app.setLoginItemSettings({ openAtLogin });
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
      log.debug(`Failed to remove path before creating a directory for it: ${error.message}`);
    }

    return mkdirAsync(directory);
  }
};
