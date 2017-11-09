// @flow
import { shell } from 'electron';

var open = (link) => {
      shell.openExternal(link);
};
export default open;