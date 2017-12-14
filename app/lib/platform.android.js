// @flow
import { BackHandler } from 'react-native';
import { Linking } from 'react-native';

const exit = () => {
  BackHandler.exitApp();
};

const open = (link: string) => {
  Linking.openURL(link);
};

export {exit, open};
