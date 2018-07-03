// @flow
import { remote, shell } from 'electron';
import electronLog from 'electron-log';

const log = electronLog;

const getAppVersion = () => {
  return remote.app.getVersion();
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

export { log, exit, openLink, openItem, getAppVersion };
