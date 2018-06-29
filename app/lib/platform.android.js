// @flow
import { BackHandler, Linking } from 'react-native';
import { MobileAppBridge } from 'NativeModules';
import { version } from '../../package.json';

const log = console.log;

const getAppVersion = () => {
  return version;
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

export { log, exit, openLink, openItem, getAppVersion };
