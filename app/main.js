// @flow
import path from 'path';
import fs from 'fs';
import log from 'electron-log';
import { app, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';
import TrayIconManager from './lib/tray-icon-manager';

import type { TrayIconType } from './lib/tray-icon-manager';

const isDevelopment = (process.env.NODE_ENV === 'development');
const isMacOS = (process.platform === 'darwin');

const appDelegate = {
  _window: (null: ?BrowserWindow),
  _tray: (null: ?Tray),

  setup: () => {
    // Override appData path to avoid collisions with old client
    // New userData path, i.e on macOS: ~/Library/Application Support/mullvad.vpn
    app.setPath('userData', path.join(app.getPath('appData'), 'mullvad.vpn'));

    if (isDevelopment) {
      log.transports.console.level = 'debug';

      // Disable log file in development
      log.transports.file.level = false;
    } else {
      log.transports.console.level = 'info';
      log.transports.file.level = 'info';
    }

    if (isDevelopment) {
      appDelegate._startBackend();
    }

    app.on('window-all-closed', () => appDelegate.onAllWindowsClosed());
    app.on('ready', () => appDelegate.onReady());
  },

  onReady: async () => {
    const window = appDelegate._window = appDelegate._createWindow();

    ipcMain.on('on-browser-window-ready', () => appDelegate._sendBackendInfo(window));

    window.loadURL('file://' + path.join(__dirname, 'index.html'));

    // create tray icon on macOS
    if(isMacOS) {
      appDelegate._tray = appDelegate._createTray(window);
    }

    appDelegate._setAppMenu();
    appDelegate._addContextMenu(window);

    if(isDevelopment) {
      await appDelegate._installDevTools();
      window.openDevTools({ mode: 'detach' });
    }
  },

  onAllWindowsClosed: () => {
    app.quit();
  },

  _startBackend: () => {
    const sudo = require('sudo-prompt');
    const pathToBackend = path.resolve(process.env.MULLVAD_BACKEND || '../talpid_core/target/debug/talpid_daemon');
    log.info('Starting the mullvad backend at', pathToBackend);

    const options = {
      name: 'mullvad backend',
    };

    sudo.exec(pathToBackend, options, (err) => {
      if (err && err.signal !== 'SIGINT') {
        log.info('Backend exited with error', err);
      } else {
        log.info('Backend exited');
      }
    });
  },

  _sendBackendInfo: (window: BrowserWindow) => {
    const file = './.mullvad_rpc_address';
    log.info('reading the ipc connection info from', file);

    fs.readFile(file, 'utf8', function (err,data) {
      if (err) {
        return log.info('Could not find backend connection info', err);
      }

      log.info('Read IPC connection info', data);
      window.webContents.send('backend-info', {
        addr: data,
      });
    });
  },

  _installDevTools: async () => {
    const installer = require('electron-devtools-installer');
    const extensions = ['REACT_DEVELOPER_TOOLS', 'REDUX_DEVTOOLS'];
    const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
    for(const name of extensions) {
      try {
        await installer.default(installer[name], forceDownload);
      } catch (e) {
        log.info(`Error installing ${name} extension: ${e.message}`);
      }
    }
  },

  _createWindow: (): BrowserWindow => {
    const contentHeight = 568;
    let options = {
      width: 320,
      height: contentHeight,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      webPreferences: {
        // prevents renderer process code from not running when window is hidden
        backgroundThrottling: false,
        // Enable experimental features
        blinkFeatures: 'CSSBackdropFilter'
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

    return new BrowserWindow(options);
  },

  _setAppMenu: () => {
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
  },

  _addContextMenu: (window: BrowserWindow) => {
    let menuTemplate = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectall' }
    ];

    // add inspect element on right click menu
    window.webContents.on('context-menu', (_e: Event, props: { x: number, y: number }) => {
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
  },

  _toggleWindow: (window: BrowserWindow, tray: ?Tray) => {
    if(window.isVisible()) {
      window.hide();
    } else {
      appDelegate._showWindow(window, tray);
    }
  },

  _showWindow: (window: BrowserWindow, tray: ?Tray) => {
    // position window based on tray icon location
    if(tray) {
      const { x, y } = appDelegate._getWindowPosition(window, tray);
      window.setPosition(x, y, false);
    }

    window.show();
    window.focus();
  },

  _getWindowPosition: (window: BrowserWindow, tray: Tray): { x: number, y: number } => {
    const windowBounds = window.getBounds();
    const trayBounds = tray.getBounds();

    // center window horizontally below the tray icon
    const x = Math.round(trayBounds.x + (trayBounds.width / 2) - (windowBounds.width / 2));

    // position window vertically below the tray icon
    const y = Math.round(trayBounds.y + trayBounds.height);

    return { x, y };
  },

  _createTray: (window: BrowserWindow): Tray => {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setHighlightMode('never');
    tray.on('click', () => appDelegate._toggleWindow(window, tray));

    // setup NSEvent monitor to fix inconsistent window.blur
    // see https://github.com/electron/electron/issues/8689
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    const trayIconManager = new TrayIconManager(tray, 'unsecured');
    const macEventMonitor = new NSEventMonitor();
    const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;

    // add IPC handler to change tray icon from renderer
    ipcMain.on('changeTrayIcon', (_: Event, type: TrayIconType) => trayIconManager.iconType = type);

    // setup event handlers
    window.on('show', () => macEventMonitor.start(eventMask, () => window.hide()));
    window.on('hide', () => macEventMonitor.stop());
    window.on('close', () => window.closeDevTools());
    window.on('blur', () => !window.isDevToolsOpened() && window.hide());

    return tray;
  }
};

appDelegate.setup();
