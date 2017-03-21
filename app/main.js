import path from 'path';
import { app, crashReporter, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';
import TrayIconManager from './lib/tray-icon-manager';

// Override appData path to avoid collisions with old client
// New userData path, i.e on macOS: ~/Library/Application Support/mullvad.vpn
const applicationSupportPath = app.getPath('appData');
const userDataPath = path.join(applicationSupportPath, 'mullvad.vpn');
app.setPath('userData', userDataPath);

const isDevelopment = (process.env.NODE_ENV === 'development');

let window = null;
let tray = null;
let macEventMonitor = null;
let trayIconManager = null;

const startTrayEventMonitor = (win) => {
  if(process.platform === 'darwin') {
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    if(macEventMonitor === null) {
      macEventMonitor = new NSEventMonitor();
    }
    macEventMonitor.start((NSEventMask.leftMouseDown | NSEventMask.rightMouseDown), () => win.hide());
  }
};

const stopTrayEventMonitor = () => {
  if(process.platform === 'darwin') {
    macEventMonitor.stop();
  }
};

ipcMain.on('changeTrayIcon', (event, type) => {
  trayIconManager.iconType = type;
});

ipcMain.emit();

// hide dock icon
if(process.platform === 'darwin') {
  app.dock.hide();
}

const installExtensions = async () => {
  const installer = require('electron-devtools-installer');
  const extensions = [
    'REACT_DEVELOPER_TOOLS',
    'REDUX_DEVTOOLS'
  ];
  const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
  for(const name of extensions) {
    try {
      await installer.default(installer[name], forceDownload);
    } catch (e) {
      console.log(`Error installing ${name} extension: ${e.message}`);
    }
  }
};

const installDevTools = async () => {
  await installExtensions();

  // show devtools when ctrl clicked
  tray.on('click', function () {
    if(!window) { return; }

    if(window.isDevToolsOpened()) {
      // there is a rare bug when isDevToolsOpened() reports true
      // but dev tools window is not created yet.
      if(window.devToolsWebContents) {
        window.devToolsWebContents.focus();
      }
    } else {
      window.openDevTools({ mode: 'detach' });
    }
  });
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

const createWindow = () => {
  window = new BrowserWindow({
    width: 320, 
    height: 568 + 12, // 12 is the size of transparent area around arrow
    frame: false,
    resizable: false,
    maximizable: false,
    fullscreenable: false,
    transparent: true,
    show: false,
    webPreferences: {
      // prevents renderer process code from not running when window is hidden
      backgroundThrottling: false,

      // Enable experimental features
      blinkFeatures: ['CSSBackdropFilter'].join(',')
    }
  });

  window.loadURL('file://' + path.join(__dirname, 'index.html'));

  // hide the window when it loses focus
  window.on('blur', () => {
    if(!window.webContents.isDevToolsOpened()) {
      window.hide();
    }
  });

  window.on('show', () => {
    startTrayEventMonitor(window);
  });

  window.on('hide', () => {
    stopTrayEventMonitor();
  });
};

const createAppMenu = () => {
  // Create the Application's main menu
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

      // mixin "inspect element" into standard menu 
      // when in development mode
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
  const position = getWindowPosition();
  window.setPosition(position.x, position.y, false);
  window.show();
  window.focus();
};

const createTray = () => {
  tray = new Tray(nativeImage.createEmpty());
  tray.on('click', toggleWindow);
  tray.setHighlightMode('never');
  
  trayIconManager = new TrayIconManager(tray);
};

crashReporter.start({
  productName: 'YourName',
  companyName: 'YourCompany',
  submitURL: 'https://your-domain.com/url-to-submit',
  uploadToServer: false
});

app.on('window-all-closed', () => {
  app.quit();
});

app.on('ready', () => {
  createTray();
  createWindow();
  createAppMenu();
  createContextMenu();

  if(isDevelopment) {
    installDevTools();
  }
});
