import os from 'os';
import path from 'path';

import { fileExists } from '../../../utils';

export const getAutoStartPath = () => {
  return path.join(os.homedir(), '.config', 'autostart', 'mullvad-vpn.desktop');
};

export const autoStartPathExists = () => {
  return fileExists(getAutoStartPath());
};
