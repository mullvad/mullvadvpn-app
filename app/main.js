// @flow
import path from 'path';
import fs from 'fs';
import mkdirp from 'mkdirp';
import { log } from './lib/platform';
import { app, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';
import TrayIconManager from './lib/tray-icon-manager';
import ElectronSudo from 'electron-sudo';
import shellescape from 'shell-escape';
import { version } from '../package.json';
import { parseIpcCredentials } from './lib/backend';
import { resolveBin } from './lib/proc';
import { execFile } from 'child_process';
import uuid from 'uuid';

import type { TrayIconType } from './lib/tray-icon-manager';

const isDevelopment = (process.env.NODE_ENV === 'development');
const isMacOS = (process.platform === 'darwin');
const isLinux = (process.platform === 'linux');

// The name for application directory used for
// scoping logs and user data in platform special folders
const appDirectoryName = 'MullvadVPN';

const writableDirectory = isMacOS || isLinux
  ? '/tmp'
  : app.getPath('temp');

const rpcAddressFile = path.join(writableDirectory, '.mullvad_rpc_address');

let browserWindowReady = false;

const appDelegate = {
  _window: (null: ?BrowserWindow),
  _tray: (null: ?Tray),
  _logFileLocation: '',
  connectionFilePollInterval: (null: ?number),

  setup: () => {
    // Override userData path, i.e on macOS: ~/Library/Application Support/MullvadVPN
    app.setPath('userData', path.join(app.getPath('appData'), appDirectoryName));

    appDelegate._logFileLocation = appDelegate._getLogsDirectory();
    appDelegate._initLogging();

    log.info('Running version', version);

    appDelegate._startBackend();

    app.on('window-all-closed', () => appDelegate.onAllWindowsClosed());
    app.on('ready', () => appDelegate.onReady());
  },

  _initLogging: () => {

    const format = '[{y}-{m}-{d} {h}:{i}:{s}.{ms}][{level}] {text}';
    log.transports.console.format = format;
    log.transports.file.format = format;
    if (isDevelopment) {
      log.transports.console.level = 'debug';

      // Disable log file in development
      log.transports.file.level = false;
    } else {
      log.transports.console.level = 'debug';
      log.transports.file.level = 'debug';
      log.transports.file.file = path.join(appDelegate._logFileLocation, 'frontend.log');
    }

    // create log folder
    mkdirp.sync(appDelegate._logFileLocation);
  },

  // Returns platform specific logs folder for application
  // See open issue and PR on Github:
  // 1. https://github.com/electron/electron/issues/10118
  // 2. https://github.com/electron/electron/pull/10191
  _getLogsDirectory: () => {
    // macOS: ~/Library/Logs/{appname}
    if(isMacOS) {
      return path.join(app.getPath('home'), 'Library/Logs', appDirectoryName);
    }
    // Linux: ~/.config/{appname}/logs
    // Windows: ~\AppData\Roaming\{appname}\logs
    return path.join(app.getPath('userData'), 'logs');
  },

  onReady: async () => {
    const window = appDelegate._window = appDelegate._createWindow();

    ipcMain.on('on-browser-window-ready', () => {
      browserWindowReady = true;
      appDelegate._pollForConnectionInfoFile();
    });

    ipcMain.on('show-window', () => {
      appDelegate._showWindow(window, appDelegate._tray);
    });

    window.loadURL('file://' + path.join(__dirname, 'index.html'));
    window.on('close', () => {
      log.debug('The browser window is closing, shutting down the tunnel...');
      window.webContents.send('shutdown');
    });

    // create tray icon on macOS
    if(isMacOS) {
      appDelegate._tray = appDelegate._createTray(window);
    } else {
      appDelegate._showWindow(window, null);
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
    const backendIsRunning = appDelegate._rpcAddressFileExists();
    if (backendIsRunning) {
      log.info('Not starting the backend as it appears to already be running');
      return;
    }

    const pathToBackend = resolveBin('mullvad-daemon');
    log.info('Starting the mullvad backend at', pathToBackend);

    const options = {
      name: 'Mullvad',
    };
    const sudo = new ElectronSudo(options);
    const backendCommand = shellescape([
      pathToBackend, '-v',
      '--log', path.join(appDelegate._logFileLocation, 'backend.log'),
      '--tunnel-log', path.join(appDelegate._logFileLocation, 'openvpn.log')
    ]);
    sudo.spawn(backendCommand, [])
      .then( p => {
        appDelegate._setupBackendProcessListeners(p);
        return p;
      });
  },
  _rpcAddressFileExists: () => {
    return fs.existsSync(rpcAddressFile);
  },
  _setupBackendProcessListeners: (p) => {
    // electron-sudo writes all output to some buffers in memory.
    // For long-running processes such as this one that would
    // cause a memory leak.
    p.stdout.removeAllListeners('data');
    p.stderr.removeAllListeners('data');

    p.stdout.on('data', (data) => {
      console.log('BACKEND stdout:', data.toString());
    });
    p.stderr.on('data', (data) => {
      console.warn('BACKEND stderr:', data.toString());
    });

    p.on('error', (err) => {
      log.error('Failed to start or kill the backend', err);
    });

    p.on('exit', (code) => {
      const timeoutMs = 500;
      log.info('The backend exited with code', code + '. Attempting to restart it in', timeoutMs, 'milliseconds...');
      setTimeout( () => appDelegate._startBackend(), timeoutMs);
    });
  },
  _pollForConnectionInfoFile: () => {

    if (appDelegate.connectionFilePollInterval) {
      log.warn('Attempted to start polling for the RPC connection info file while another polling was already running');
      return;
    }

    const pollIntervalMs = 200;
    appDelegate.connectionFilePollInterval = setInterval(() => {

      if (browserWindowReady && appDelegate._rpcAddressFileExists()) {

        if (appDelegate.connectionFilePollInterval) {
          clearInterval(appDelegate.connectionFilePollInterval);
          appDelegate.connectionFilePollInterval = null;
        }

        appDelegate._sendBackendInfo();
      }

    }, pollIntervalMs);
  },
  _sendBackendInfo: () => {
    const window = appDelegate._window;
    if (!window) {
      log.error('Attempted to send backend rpc address before the window was ready');
      return;
    }

    log.debug('Reading the ipc connection info from', rpcAddressFile);

    const isSecureEnough = isOwnedAndOnlyWritableByRoot(rpcAddressFile);
    if (!isSecureEnough) {
      log.error('Not trusting the contents of', rpcAddressFile, 'as it was not owned and only writable by root.');
      return;
    }

    // There is a race condition here where the owner and permissions of
    // the file can change in the time between we validate the owner and
    // permissions and read the contents of the file. We deem the chance
    // of that to be small enough to ignore.

    fs.readFile(rpcAddressFile, 'utf8', function (err, data) {
      if (err) {
        return log.error('Could not find backend connection info', err);
      }

      const credentials = parseIpcCredentials(data);
      if(credentials) {
        log.debug('Read IPC connection info', credentials.connectionString);
        window.webContents.send('backend-info', { credentials });
      } else {
        log.error('Could not parse IPC credentials.');
      }
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
    const options = {
      width: 320,
      height: contentHeight,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      show: false,
      webPreferences: {
        // prevents renderer process code from not running when window is hidden
        backgroundThrottling: false,
        // Enable experimental features
        blinkFeatures: 'CSSBackdropFilter'
      }
    };

    // setup window flags to mimic popover on macOS
    if(isMacOS) {
      const win = new BrowserWindow({
        ...options,
        height: contentHeight + 12, // 12 is the size of transparent area around arrow
        frame: false,
        transparent: true
      });
      win.setVisibleOnAllWorkspaces(true);
      return win;
    } else {
      return new BrowserWindow(options);
    }
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

    ipcMain.on('collect-logs', (event, id, toRedact) => {
      log.info('Collecting logs in', appDelegate._logFileLocation);
      fs.readdir(appDelegate._logFileLocation, (err, files) => {
        if (err) {
          event.sender.send('collect-logs-reply', id, err);
          return;
        }

        const logFiles = files.filter(file => file.endsWith('.log'))
          .map(f => path.join(appDelegate._logFileLocation, f));
        const reportPath = path.join(writableDirectory, uuid.v4() + '.report');

        const binPath = resolveBin('problem-report');
        let args = [
          'collect',
          '--output', reportPath,
        ];

        if (toRedact.length > 0) {
          args = args.concat([
            '--redact', ...toRedact,
            '--',
          ]);
        }

        args = args.concat(logFiles);

        execFile(binPath, args, {windowsHide: true}, (err) => {
          if (err) {
            event.sender.send('collect-logs-reply', id, err);
          } else {
            log.debug('Report written to', reportPath);
            event.sender.send('collect-logs-reply', id, null, reportPath);
          }
        });
      });
    });

    // setup event handlers
    window.on('show', () => macEventMonitor.start(eventMask, () => window.hide()));
    window.on('hide', () => macEventMonitor.stop());
    window.on('close', () => window.closeDevTools());
    window.on('blur', () => !window.isDevToolsOpened() && window.hide());

    return tray;
  }
};

appDelegate.setup();

function isOwnedAndOnlyWritableByRoot(path) {
  const stat = fs.statSync(path);
  const isOwnedByRoot = stat.uid === 0;
  const isOnlyWritableByOwner = (stat.mode & parseInt('022', 8)) === 0;

  return isOwnedByRoot && isOnlyWritableByOwner;
}
