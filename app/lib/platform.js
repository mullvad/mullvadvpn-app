// @flow
import fs from 'fs';
import { remote, shell } from 'electron';
import electronLog from 'electron-log';
import { execFile } from 'child_process';
import path from 'path';
import { promisify } from 'util';

const desktopFileName = 'mullvad-vpn.desktop';

const execFileAsync = promisify(execFile);
const fsMkdirAsync = promisify(fs.mkdir);
const fsStatAsync = promisify(fs.stat);
const fsSymlinkAsync = promisify(fs.symlink);
const fsUnlinkAsync = promisify(fs.unlink);

const log = electronLog;

const getAppVersion = () => {
  return remote.app.getVersion();
};

const getOpenAtLogin = (): boolean => {
  if (process.platform === 'linux') {
    try {
      const autostartDir = getXdgAutostartDirSync();
      const autostartFilePath = path.join(autostartDir, desktopFileName);

      fs.accessSync(autostartFilePath);

      return true;
    } catch (error) {
      log.debug(`Failed to check autostart file: ${error.message}`);
      return false;
    }
  } else {
    return remote.app.getLoginItemSettings().openAtLogin;
  }
};

const setOpenAtLogin = async (openAtLogin: boolean) => {
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
      const desktopFilePath = path.join('/usr/share/applications', desktopFileName);
      const autostartDir = await getXdgAutostartDir();
      const autostartFilePath = path.join(autostartDir, desktopFileName);

      if (openAtLogin) {
        await fsSymlinkAsync(desktopFilePath, autostartFilePath);
      } else {
        await fsUnlinkAsync(autostartFilePath);
      }
    } catch (error) {
      log.error(`Failed to set auto-start: ${error.message}`);
    }
  } else {
    remote.app.setLoginItemSettings({ openAtLogin });
  }
};

const getXdgAutostartDir = async (): Promise<string> => {
  const configDir = await getXdgHomeDir();
  const autostartDir = path.join(configDir, 'autostart');

  if (autostartDir) {
    await createDirIfNecessary(autostartDir);
    return autostartDir;
  } else {
    throw new Error('No XDG autostart directory found');
  }
};

const getXdgAutostartDirSync = (): string => {
  const configDir = getXdgHomeDirSync();
  const autostartDir = path.join(configDir, 'autostart');

  if (autostartDir) {
    return autostartDir;
  } else {
    throw new Error('No XDG autostart directory found');
  }
};

const getXdgHomeDir = async (): Promise<string> => {
  const xdgConfigHome = process.env.XDG_CONFIG_HOME;

  if (xdgConfigHome && (await isDirectory(xdgConfigHome))) {
    return Promise.resolve(xdgConfigHome);
  } else {
    const home = process.env.HOME;

    if (home == null || !(await isDirectory(home))) {
      throw new Error("Home path doesn't exist");
    }

    const configHome = path.join(home, '.config');
    await createDirIfNecessary(configHome);

    return Promise.resolve(configHome);
  }
};

const getXdgHomeDirSync = (): string => {
  const xdgConfigHome = process.env.XDG_CONFIG_HOME;

  if (xdgConfigHome && isDirectorySync(xdgConfigHome)) {
    return xdgConfigHome;
  } else {
    const home = process.env.HOME;

    if (home == null || !isDirectorySync(home)) {
      throw new Error("Home path doesn't exist");
    }

    return path.join(home, '.config');
  }
};

const createDirIfNecessary = async (directory: string) => {
  if (!(await isDirectory(directory))) {
    try {
      await fsUnlinkAsync(directory);
    } catch (error) {
      log.silly(`Failed to remove path before creating a directory for it: ${error.message}`);
    }
    await fsMkdirAsync(directory);
  }
};

const isDirectory = async (directory: ?string): Promise<boolean> => {
  if (directory) {
    try {
      const stat = await fsStatAsync(directory);
      return Promise.resolve(stat.isDirectory());
    } catch (error) {
      // Failed to stat directory
    }
  }
  return Promise.resolve(false);
};

const isDirectorySync = (directory: ?string): boolean => {
  if (directory) {
    try {
      const stat = fs.statSync(directory);
      return stat.isDirectory();
    } catch (error) {
      // Failed to stat directory
    }
  }
  return false;
};

const exit = () => {
  remote.app.quit();
};

const openLink = (link: string) => {
  shell.openExternal(link);
};

const openItem = (path: string) => {
  shell.openItem(path);
};

export { log, exit, openLink, openItem, getAppVersion, getOpenAtLogin, setOpenAtLogin };
