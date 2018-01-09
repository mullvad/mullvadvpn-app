// @flow
import { BackHandler, Linking } from 'react-native';

const log = console.log;

const exit = () => {
  BackHandler.exitApp();
};

const openLink = (link: string) => {
  Linking.openURL(link);
};

const openItem = (path: string) => {

};

export { log, exit, openLink, openItem };



