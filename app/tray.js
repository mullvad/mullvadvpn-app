import path from 'path';
import { app, Menu, Tray } from 'electron';

class TrayMenu {

  trayImpl = null;

  setup() {
    const iconPath = path.join(__dirname, "assets/trayicon.png");
    this.trayImpl = new Tray(iconPath);

    const contextMenu = Menu.buildFromTemplate([
      {label: 'Item1', type: 'radio'},
      {label: 'Item2', type: 'radio'},
      {label: 'Item3', type: 'radio', checked: true},
      {label: 'Item4', type: 'radio'}
    ]);

    this.trayImpl.setContextMenu(contextMenu);
  }
  
}

module.exports = new TrayMenu();
