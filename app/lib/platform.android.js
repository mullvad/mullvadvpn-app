// @flow
import { BackHandler, Linking } from 'react-native';
import { MobileAppBridge } from 'NativeModules';

const log = console.log;

const exit = () => {
  BackHandler.exitApp();
};

const openLink = (link: string) => {
  Linking.openURL(link);
};

const openItem = (path: string) => {
  MobileAppBridge.openItem(path);
};

export { log, exit, openLink, openItem };
