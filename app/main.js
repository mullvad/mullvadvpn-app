import path from 'path';
import fs from 'fs';
import { app, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';
import TrayIconManager from './lib/tray-icon-manager';

const isDevelopment = (process.env.NODE_ENV === 'development');
const isMacOS = (process.platform === 'darwin');

let window = null;
let tray = null;

// Override appData path to avoid collisions with old client
// New userData path, i.e on macOS: ~/Library/Application Support/mullvad.vpn
app.setPath('userData', path.join(app.getPath('appData'), 'mullvad.vpn'));

ipcMain.on('on-browser-window-ready', () => {
  sendBackendInfo();
});

const installDevTools = async () => {
  const installer = require('electron-devtools-installer');
  const extensions = ['REACT_DEVELOPER_TOOLS', 'REDUX_DEVTOOLS'];
  const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
  for(const name of extensions) {
    try {
      await installer.default(installer[name], forceDownload);
    } catch (e) {
      console.log(`Error installing ${name} extension: ${e.message}`);
    }
  }
};

const createWindow = () => {
  const contentHeight = 568;
  let options = {
    width: 320,
    height: contentHeight,
    resizable: false,
    maximizable: false,
    fullscreenable: false,
    show: true,
    webPreferences: {
      // prevents renderer process code from not running when window is hidden
      backgroundThrottling: false,
      // Enable experimental features
      blinkFeatures: ['CSSBackdropFilter'].join(',')
    }
  };

  // setup window flags to mimic popover on macOS
  if(isMacOS) {
    options = Object.assign({}, options, {
      height: contentHeight + 12, // 12 is the size of transparent area around arrow
      frame: false,
      transparent: true,
      show: false
    });
  }

  window = new BrowserWindow(options);
  window.loadURL('file://' + path.join(__dirname, 'index.html'));
};

const createAppMenu = () => {
  const template = [
    {
      label: 'Mullvad',
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'quit' }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { type: 'separator' },
        { role: 'selectall' }
      ]
    }
  ];

  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
};

const createContextMenu = () => {
  let menuTemplate = [
    { role: 'cut' },
    { role: 'copy' },
    { role: 'paste' },
    { type: 'separator' },
    { role: 'selectall' }
  ];

  // add inspect element on right click menu
  window.webContents.on('context-menu', (e, props) => {
    let inspectTemplate = [{
      label: 'Inspect element',
      click() {
        window.openDevTools({ mode: 'detach' });
        window.inspectElement(props.x, props.y);
      }
    }];

    if(props.isEditable) {
      let inputMenu = menuTemplate;

      // mixin 'inspect element' into standard menu when in development mode
      if(isDevelopment) {
        inputMenu = menuTemplate.concat([{type: 'separator'}], inspectTemplate);
      }

      Menu.buildFromTemplate(inputMenu).popup(window);
    } else if(isDevelopment) {
      // display inspect element for all non-editable
      // elements when in development mode
      Menu.buildFromTemplate(inspectTemplate).popup(window);
    }
  });
};

const toggleWindow = () => {
  if (window.isVisible()) {
    window.hide();
  } else {
    showWindow();
  }
};

const showWindow = () => {
  // position window based on tray icon location
  if(tray) {
    const { x, y } = getWindowPosition();
    window.setPosition(x, y, false);
  }

  window.show();
  window.focus();
};

const getWindowPosition = () => {
  const windowBounds = window.getBounds();
  const trayBounds = tray.getBounds();

  // center window horizontally below the tray icon
  const x = Math.round(trayBounds.x + (trayBounds.width / 2) - (windowBounds.width / 2));

  // position window vertically below the tray icon
  const y = Math.round(trayBounds.y + trayBounds.height);

  return { x, y };
};

const createTray = () => {
  tray = new Tray(nativeImage.createEmpty());
  tray.setHighlightMode('never');
  tray.on('click', toggleWindow);

  // setup NSEvent monitor to fix inconsistent window.blur
  // see https://github.com/electron/electron/issues/8689
  const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
  const trayIconManager = new TrayIconManager(tray);
  const macEventMonitor = new NSEventMonitor();
  const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;

  // add IPC handler to change tray icon from renderer
  ipcMain.on('changeTrayIcon', (_, type) => trayIconManager.iconType = type);

  // setup event handlers
  window.on('show', () => macEventMonitor.start(eventMask, () => window.hide()));
  window.on('hide', () => macEventMonitor.stop());
  window.on('close', () => window.closeDevTools());
  window.on('blur', () => !window.isDevToolsOpened() && window.hide());
};

app.on('window-all-closed', () => {
  app.quit();
});

app.on('ready', async () => {
  createWindow();

  // create tray icon on macOS
  isMacOS && createTray();

  createAppMenu();
  createContextMenu();

  if(isDevelopment) {
    await installDevTools();
    window.openDevTools({ mode: 'detach' });
  }
});

const sendBackendInfo = () => {
  const file = './.ipc_connection_info';
  console.log('reading the ipc connection info from', file);

  fs.readFile(file, 'utf8', function (err,data) {
    if (err) {
      return console.log('Could not find backend connection info', err);
    }

    console.log('Read IPC connection info', data);
    window.webContents.send('backend-info', {
      addr: data,
    });
  });
};

