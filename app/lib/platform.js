// @flow
import { remote, shell } from 'electron';
import electronLog from 'electron-log';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

const log = electronLog;

const getAppVersion = () => {
  return remote.app.getVersion();
};

const getOpenAtLogin = () => {
  return remote.app.getLoginItemSettings().openAtLogin;
};

const setOpenAtLogin = async (openAtLogin: boolean) => {
  // setLoginItemSettings is broken on macOS and cannot delete login items.
  // Issue: https://github.com/electron/electron/issues/10880
  if (process.platform === 'darwin' && openAtLogin === false) {
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
