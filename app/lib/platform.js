// @flow
import { remote } from 'electron';
import { shell } from 'electron';

const exit = () => {
  remote.app.quit();
};

const open = (link) => {
  shell.openExternal(link);
};

export {exit, open};