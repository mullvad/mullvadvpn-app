import { nativeImage, Tray } from 'electron';

export function createTray() {
  const tray = new Tray(nativeImage.createEmpty());

  tray.setToolTip('Mullvad VPN');

  // disable double click on tray icon since it causes weird delay
  tray.setIgnoreDoubleClickEvents(true);

  return tray;
}
