// @flow
import { remote } from 'electron';
import { shell } from 'electron';
import electronLog from 'electron-log';

const log = electronLog;

const exit = () => {
  remote.app.quit();
};

const openLink = (link: string) => {
  shell.openExternal(link);
};

const openItem = (path: string) => {
  shell.openItem(path);
};

export { log, exit, openLink, openItem };
