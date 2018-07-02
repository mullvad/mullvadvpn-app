// @flow
import path from 'path';
import fs from 'fs';
import mkdirp from 'mkdirp';
import { log } from './lib/platform';
import { app, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';
import TrayIconController from './tray-icon-controller';
import WindowController from './window-controller';
import { version } from '../package.json';
import { resolveBin } from './lib/proc';
import { getSystemTemporaryDirectory } from './lib/tempdir';
import { canTrustRpcAddressFile } from './lib/rpc-file-security';
import { execFile } from 'child_process';
import uuid from 'uuid';

import { ShutdownCoordinator } from './shutdown-handler';
import type { TrayIconType } from './tray-icon-controller';

/*
HOTFIX
We had an issue importing this stuff from backend.js but
since it's going away in one of already open PRs,
there is no point in trying to make it any nicer,

Hence duplicating this piece in here
TODO: Remove during merge
 */
export type IpcCredentials = {
  connectionString: string,
  sharedSecret: string,
};
export function parseIpcCredentials(data: string): ?IpcCredentials {
  const [connectionString, sharedSecret] = data.split('\n', 2);
  if (connectionString && sharedSecret !== undefined) {
    return {
      connectionString,
      sharedSecret,
    };
  } else {
    return null;
  }
}

/** / HOTFIX  */

// The name for application directory used for
// scoping logs and user data in platform special folders
const appDirectoryName = 'Mullvad VPN';

const ApplicationMain = {
  _windowController: (null: ?WindowController),
  _trayIconController: (null: ?TrayIconController),

  _logFilePath: '',
  _connectionFilePollInterval: (null: ?IntervalID),

  run() {
    if (this._ensureSingleInstance()) {
      return;
    }

    // Override userData path, i.e on macOS: ~/Library/Application Support/Mullvad VPN
    app.setPath('userData', path.join(app.getPath('appData'), appDirectoryName));
    this._initLogging();

    log.info('Running version', version);

    app.on('ready', () => this._onReady());
    app.on('window-all-closed', () => app.quit());
  },

  _ensureSingleInstance() {
    // This callback is guaranteed to be excuted after 'ready' events have been
    // sent to the app.
    const shouldQuit = app.makeSingleInstance((_args, _workingDirectory) => {
      log.debug('Another instance was spawned, showing window');

      if (this._windowController) {
        this._windowController.show();
      }
    });

    if (shouldQuit) {
      log.info('Another instance already exists, shutting down');
      app.exit();
    }

    return shouldQuit;
  },

  _initLogging() {
    const logDirectory = this._getLogsDirectory();
    const format = '[{y}-{m}-{d} {h}:{i}:{s}.{ms}][{level}] {text}';

    this._logFilePath = path.join(logDirectory, 'frontend.log');

    log.transports.console.format = format;
    log.transports.file.format = format;
    if (process.env.NODE_ENV === 'development') {
      log.transports.console.level = 'debug';

      // Disable log file in development
      log.transports.file.level = false;
    } else {
      log.transports.console.level = 'debug';
      log.transports.file.level = 'debug';
      log.transports.file.file = this._logFilePath;
    }

    // create log folder
    mkdirp.sync(logDirectory);
  },

  // Returns platform specific logs folder for application
  // See open issue and PR on Github:
  // 1. https://github.com/electron/electron/issues/10118
  // 2. https://github.com/electron/electron/pull/10191
  _getLogsDirectory() {
    switch (process.platform) {
      case 'darwin':
        // macOS: ~/Library/Logs/{appname}
        return path.join(app.getPath('home'), 'Library/Logs', appDirectoryName);
      default:
        // Windows: %APPDATA%\{appname}\logs
        // Linux: ~/.config/{appname}/logs
        return path.join(app.getPath('userData'), 'logs');
    }
  },

  async _onReady() {
    const window = this._createWindow();
    const tray = this._createTray();

    const _shutdownCoordinator = new ShutdownCoordinator(window.webContents);

    const windowController = new WindowController(window, tray);
    const trayIconController = new TrayIconController(tray, 'unsecured');

    tray.on('click', () => windowController.toggle());

    this._registerIpcListeners();
    this._setAppMenu();
    this._addContextMenu(window);

    this._windowController = windowController;
    this._trayIconController = trayIconController;

    if (process.env.NODE_ENV === 'development') {
      await this._installDevTools();

      window.on('close', () => window.closeDevTools());
      window.openDevTools({ mode: 'detach' });
    }

    if (this._isMenubarApp()) {
      this._installMenubarAppEventHandlers(windowController);
    } else {
      windowController.show();
    }

    window.loadFile('build/index.html');
  },

  _registerIpcListeners() {
    ipcMain.on('on-browser-window-ready', () => {
      this._pollConnectionInfoFile();
    });

    ipcMain.on('show-window', () => {
      const windowController = this._windowController;
      if (windowController) {
        windowController.show();
      }
    });

    ipcMain.on('hide-window', () => {
      const windowController = this._windowController;
      if (windowController) {
        windowController.hide();
      }
    });

    ipcMain.on('change-tray-icon', (_event: any, type: TrayIconType) => {
      const trayIconController = this._trayIconController;
      if (trayIconController) {
        trayIconController.animateToIcon(type);
      }
    });

    ipcMain.on('collect-logs', (event, requestId, toRedact) => {
      const reportPath = path.join(app.getPath('temp'), uuid.v4() + '.log');
      const executable = resolveBin('problem-report');
      const args = ['collect', '--output', reportPath];
      if (toRedact.length > 0) {
        args.push('--redact', ...toRedact, '--');
      }
      args.push(this._logFilePath);

      execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
        if (error) {
          log.error(
            `Failed to collect a problem report: ${error.message}
             Stdout: ${stdout.toString()}
             Stderr: ${stderr.toString()}`,
          );

          event.sender.send('collect-logs-reply', requestId, {
            success: false,
            error: error.message,
          });
        } else {
          log.debug(`Problem report was written to ${reportPath}`);

          event.sender.send('collect-logs-reply', requestId, {
            success: true,
            reportPath,
          });
        }
      });
    });

    ipcMain.on(
      'send-problem-report',
      (event, requestId, email: string, message: string, savedReport: string) => {
        const executable = resolveBin('problem-report');
        const args = ['send', '--email', email, '--message', message, '--report', savedReport];

        execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
          if (error) {
            log.error(
              `Failed to send a problem report: ${error.message}
           Stdout: ${stdout.toString()}
           Stderr: ${stderr.toString()}`,
            );

            event.sender.send('send-problem-report-reply', requestId, {
              success: false,
              error: error.message,
            });
          } else {
            log.info('Problem report was sent.');

            event.sender.send('send-problem-report-reply', requestId, {
              success: true,
            });
          }
        });
      },
    );
  },

  _getRpcAddressFilePath() {
    const rpcAddressFileName = '.mullvad_rpc_address';

    switch (process.platform) {
      case 'win32': {
        // Windows: %ALLUSERSPROFILE%\{appname}
        const programDataDirectory = process.env.ALLUSERSPROFILE;
        if (programDataDirectory) {
          const appDataDirectory = path.join(programDataDirectory, appDirectoryName);
          return path.join(appDataDirectory, rpcAddressFileName);
        } else {
          throw new Error('Missing %ALLUSERSPROFILE% environment variable');
        }
      }
      default:
        return path.join(getSystemTemporaryDirectory(), rpcAddressFileName);
    }
  },

  _pollConnectionInfoFile() {
    if (this._connectionFilePollInterval) {
      log.warn(
        'Attempted to start polling for the RPC connection info file while another polling was already running',
      );
      return;
    }

    const pollIntervalMs = 200;
    const rpcAddressFile = this._getRpcAddressFilePath();

    this._connectionFilePollInterval = setInterval(() => {
      if (fs.existsSync(rpcAddressFile)) {
        if (this._connectionFilePollInterval) {
          clearInterval(this._connectionFilePollInterval);
          this._connectionFilePollInterval = null;
        }

        this._sendDaemonConnectionInfo(rpcAddressFile);
      }
    }, pollIntervalMs);
  },

  _sendDaemonConnectionInfo(rpcAddressFile: string) {
    log.debug(`Reading the ipc connection info from "${rpcAddressFile}"`);

    try {
      if (!canTrustRpcAddressFile(rpcAddressFile)) {
        log.error(`Not trusting the contents of "${rpcAddressFile}".`);
        return;
      }
    } catch (e) {
      log.error(`Cannot verify the credibility of RPC address file: ${e.message}`);
      return;
    }

    // There is a race condition here where the owner and permissions of
    // the file can change in the time between we validate the owner and
    // permissions and read the contents of the file. We deem the chance
    // of that to be small enough to ignore.

    fs.readFile(rpcAddressFile, 'utf8', (err, data) => {
      if (err) {
        return log.error('Could not find backend connection info', err);
      }

      const credentials = parseIpcCredentials(data);
      if (credentials) {
        log.debug('Read IPC connection info', credentials.connectionString);
        const windowController = this._windowController;
        if (windowController) {
          windowController.window.webContents.send('backend-info', { credentials });
        }
      } else {
        log.error('Could not parse IPC credentials.');
      }
    });
  },

  async _installDevTools() {
    const installer = require('electron-devtools-installer');
    const extensions = ['REACT_DEVELOPER_TOOLS', 'REDUX_DEVTOOLS'];
    const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
    for (const name of extensions) {
      try {
        await installer.default(installer[name], forceDownload);
      } catch (e) {
        log.info(`Error installing ${name} extension: ${e.message}`);
      }
    }
  },

  _createWindow(): BrowserWindow {
    const contentHeight = 568;

    // the size of transparent area around arrow on macOS
    const headerBarArrowHeight = 12;

    const options = {
      width: 320,
      minWidth: 320,
      height: contentHeight,
      minHeight: contentHeight,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      show: false,
      frame: false,
      webPreferences: {
        // prevents renderer process code from not running when window is hidden
        backgroundThrottling: false,
        // Enable experimental features
        blinkFeatures: 'CSSBackdropFilter',
      },
    };

    switch (process.platform) {
      case 'darwin': {
        // setup window flags to mimic popover on macOS
        const appWindow = new BrowserWindow({
          ...options,
          height: contentHeight + headerBarArrowHeight,
          minHeight: contentHeight + headerBarArrowHeight,
          transparent: true,
        });

        // make the window visible on all workspaces
        appWindow.setVisibleOnAllWorkspaces(true);

        return appWindow;
      }

      case 'win32':
        // setup window flags to mimic an overlay window
        return new BrowserWindow({
          ...options,
          transparent: true,
        });

      default:
        return new BrowserWindow(options);
    }
  },

  _setAppMenu() {
    const template = [
      {
        label: 'Mullvad',
        submenu: [{ role: 'about' }, { type: 'separator' }, { role: 'quit' }],
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
          { type: 'separator' },
          { role: 'selectall' },
        ],
      },
    ];
    Menu.setApplicationMenu(Menu.buildFromTemplate(template));
  },

  _addContextMenu(window: BrowserWindow) {
    const menuTemplate = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectall' },
    ];

    // add inspect element on right click menu
    window.webContents.on(
      'context-menu',
      (_e: Event, props: { x: number, y: number, isEditable: boolean }) => {
        const inspectTemplate = [
          {
            label: 'Inspect element',
            click() {
              window.openDevTools({ mode: 'detach' });
              window.inspectElement(props.x, props.y);
            },
          },
        ];

        if (props.isEditable) {
          let inputMenu = menuTemplate;

          // mixin 'inspect element' into standard menu when in development mode
          if (process.env.NODE_ENV === 'development') {
            inputMenu = menuTemplate.concat([{ type: 'separator' }], inspectTemplate);
          }

          Menu.buildFromTemplate(inputMenu).popup(window);
        } else if (process.env.NODE_ENV === 'development') {
          // display inspect element for all non-editable
          // elements when in development mode
          Menu.buildFromTemplate(inspectTemplate).popup(window);
        }
      },
    );
  },

  _createTray(): Tray {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setToolTip('Mullvad VPN');

    // disable icon highlight on macOS
    if (process.platform === 'darwin') {
      tray.setHighlightMode('never');
    }

    return tray;
  },

  _isMenubarApp() {
    const platform = process.platform;

    return platform === 'windows' || platform === 'darwin';
  },

  _installMenubarAppEventHandlers(windowController: WindowController) {
    switch (process.platform) {
      case 'windows':
        windowController.window.on('blur', () => windowController.hide());
        break;

      case 'darwin':
        this._installMacOsMenubarAppWindowHandlers(windowController);
        break;

      default:
        break;
    }
  },

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  _installMacOsMenubarAppWindowHandlers(windowController: WindowController) {
    // $FlowFixMe: this module is only available on macOS
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    const macEventMonitor = new NSEventMonitor();
    const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;
    const window = windowController.window;

    window.on('show', () => macEventMonitor.start(eventMask, () => windowController.hide()));
    window.on('hide', () => macEventMonitor.stop());
  },
};

ApplicationMain.run();
