import { startApp } from '../utils';

export const startInstalledApp = async (): ReturnType<typeof startApp> => {
  return startApp({ executablePath: getAppInstallPath() });
}

function getAppInstallPath(): string {
  switch (process.platform) {
    case 'win32':
      return 'C:\\Program Files\\Mullvad VPN\\Mullvad VPN.exe';
    case 'linux':
      return '/opt/Mullvad VPN/mullvad-gui';
    case 'darwin':
      return '/Applications/Mullvad VPN.app/Contents/MacOS/Mullvad VPN';
    default:
      throw new Error('Platform not supported');
  }
}
