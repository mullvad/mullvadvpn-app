// @flow
import { Linking } from 'react-native';

var open = (link) => {
	Linking.openURL(link);
};
export default open;
