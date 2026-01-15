import { Tray } from 'electron';

import { TrayIcon } from './tray-icon';

function getInitialIcon() {
  if (process.platform === 'linux') {
    return new TrayIcon('lock-placeholder');
  }

  return new TrayIcon();
}

export function createTray() {
  const initialIcon = getInitialIcon();

  const tray = new Tray(initialIcon.toNativeImage());

  tray.setToolTip('Mullvad VPN');

  // disable double click on tray icon since it causes weird delay
  tray.setIgnoreDoubleClickEvents(true);

  return tray;
}
