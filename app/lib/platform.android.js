// @flow
import { BackHandler, Linking } from 'react-native';
import { MobileAppBridge } from 'NativeModules';
import { version } from '../../package.json';

const log = console.log;

const getAppVersion = () => {
  return version;
};

const getOpenAtLogin = () => {
  throw new Error('Not implemented');
};

const setOpenAtLogin = (_autoStart: boolean) => {
  throw new Error('Not implemented');
};

const exit = () => {
  BackHandler.exitApp();
};

const openLink = (link: string) => {
  Linking.openURL(link);
};

const openItem = (path: string) => {
  MobileAppBridge.openItem(path);
};

export { log, exit, openLink, openItem, getAppVersion, getOpenAtLogin, setOpenAtLogin };
