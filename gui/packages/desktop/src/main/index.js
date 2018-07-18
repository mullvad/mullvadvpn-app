// @flow

import fs from 'fs';
import log from 'electron-log';
import path from 'path';
import { execFile } from 'child_process';
import mkdirp from 'mkdirp';
import uuid from 'uuid';
import { app, screen, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';

import TrayIconController from './tray-icon-controller';
import WindowController from './window-controller';
import RpcAddressFile from './rpc-address-file';
import ShutdownCoordinator from './shutdown-coordinator';
import { resolveBin } from './proc';

import type { TrayIconType } from './tray-icon-controller';

const ApplicationMain = {
  _windowController: (null: ?WindowController),
  _trayIconController: (null: ?TrayIconController),

  _logFilePath: '',
  _oldLogFilePath: (null: ?string),
  _connectionFilePollInterval: (null: ?IntervalID),
  _shouldQuit: false,

  run() {
    this._overrideAppPaths();

    if (this._ensureSingleInstance()) {
      return;
    }

    this._initLogging();

    log.info(`Running version ${app.getVersion()}`);

    if (process.platform === 'win32') {
      app.setAppUserModelId('net.mullvad.vpn');
    }

    app.on('activate', () => this._onActivate());
    app.on('ready', () => this._onReady());
    app.on('window-all-closed', () => app.quit());
    app.on('before-quit', () => this._onBeforeQuit());
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

  _overrideAppPaths() {
    // This ensures that on Windows the %LOCALAPPDATA% directory is used instead of the %ADDDATA%
    // directory that has roaming contents
    if (process.platform === 'win32') {
      const appDataDir = process.env.LOCALAPPDATA;
      if (appDataDir) {
        app.setPath('appData', appDataDir);
        app.setPath('userData', path.join(appDataDir, app.getName()));
      } else {
        throw new Error('Missing %LOCALAPPDATA% environment variable');
      }
    }
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
      // Create log folder
      mkdirp.sync(logDirectory);

      // Backup previous log file if it exists
      try {
        fs.accessSync(this._logFilePath);
        this._oldLogFilePath = path.join(logDirectory, 'frontend.old.log');
        fs.renameSync(this._logFilePath, this._oldLogFilePath);
      } catch (error) {
        // No previous log file exists
      }

      // Configure logging to file
      log.transports.console.level = 'debug';
      log.transports.file.level = 'debug';
      log.transports.file.file = this._logFilePath;

      log.debug(`Logging to ${this._logFilePath}`);
    }
  },

  // Returns platform specific logs folder for application
  // See open issue and PR on Github:
  // 1. https://github.com/electron/electron/issues/10118
  // 2. https://github.com/electron/electron/pull/10191
  _getLogsDirectory() {
    switch (process.platform) {
      case 'darwin':
        // macOS: ~/Library/Logs/{appname}
        return path.join(app.getPath('home'), 'Library/Logs', app.getName());
      default:
        // Windows: %LOCALAPPDATA%\{appname}\logs
        // Linux: ~/.config/{appname}/logs
        return path.join(app.getPath('userData'), 'logs');
    }
  },

  _onActivate() {
    const windowController = this._windowController;
    if (windowController) {
      windowController.show();
    }
  },

  _onBeforeQuit() {
    this._shouldQuit = true;

    if (process.env.NODE_ENV === 'development') {
      const windowController = this._windowController;
      if (windowController) {
        windowController.window.closeDevTools();
      }
    }

    return true;
  },

  async _onReady() {
    const window = this._createWindow();
    const tray = this._createTray();

    const _shutdownCoordinator = new ShutdownCoordinator(window.webContents);

    const windowController = new WindowController(window, tray);
    const trayIconController = new TrayIconController(tray, 'unsecured');

    this._registerIpcListeners();
    this._setAppMenu();
    this._addContextMenu(window);

    this._windowController = windowController;
    this._trayIconController = trayIconController;

    if (process.env.NODE_ENV === 'development') {
      await this._installDevTools();
      window.openDevTools({ mode: 'detach' });
    }

    switch (process.platform) {
      case 'win32':
        this._installWindowsMenubarAppWindowHandlers(tray, windowController);
        break;
      case 'darwin':
        this._installMacOsMenubarAppWindowHandlers(tray, windowController);
        break;
      case 'linux':
        this._installGenericMenubarAppWindowHandlers(tray, windowController);
        this._installLinuxWindowCloseHandler(windowController);
        break;
      default:
        this._installGenericMenubarAppWindowHandlers(tray, windowController);
        break;
    }

    window.loadFile(path.resolve(path.join(__dirname, '../renderer/index.html')));
  },

  _registerIpcListeners() {
    ipcMain.on('discover-daemon-connection', async (event) => {
      const addressFile = new RpcAddressFile();

      log.debug(`Waiting for RPC address file: "${addressFile.filePath}"`);

      try {
        await addressFile.waitUntilExists();
      } catch (error) {
        log.error(`Cannot finish polling the RPC address file: ${error.message}`);
        return;
      }

      try {
        if (!addressFile.isTrusted()) {
          log.error(`Cannot verify the credibility of RPC address file`);
          return;
        }
      } catch (error) {
        log.error(`An error occurred during the credibility check: ${error.message}`);
        return;
      }

      // There is a race condition here where the owner and permissions of
      // the file can change in the time between we validate the owner and
      // permissions and read the contents of the file. We deem the chance
      // of that to be small enough to ignore.

      try {
        const credentials = await addressFile.parse();

        log.debug('Read RPC connection info', credentials.connectionString);

        event.sender.send('daemon-connection-ready', credentials);
      } catch (error) {
        log.error(`Cannot parse the RPC address file: ${error.message}`);
        return;
      }
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
      if (this._oldLogFilePath) {
        args.push(this._oldLogFilePath);
      }

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
          skipTaskbar: true,
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

  _installWindowsMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    tray.on('click', () => windowController.toggle());
    tray.on('right-click', () => windowController.hide());

    windowController.window.on('blur', () => {
      // Detect if blur happened when user had a cursor above the tray icon.
      const trayBounds = tray.getBounds();
      const cursorPos = screen.getCursorScreenPoint();
      const isCursorInside =
        cursorPos.x >= trayBounds.x &&
        cursorPos.y >= trayBounds.y &&
        cursorPos.x <= trayBounds.x + trayBounds.width &&
        cursorPos.y <= trayBounds.y + trayBounds.height;
      if (!isCursorInside) {
        windowController.hide();
      }
    });
  },

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  _installMacOsMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    // $FlowFixMe: this module is only available on macOS
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    const macEventMonitor = new NSEventMonitor();
    const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;
    const window = windowController.window;

    window.on('show', () => macEventMonitor.start(eventMask, () => windowController.hide()));
    window.on('hide', () => macEventMonitor.stop());
    tray.on('click', () => windowController.toggle());
  },

  _installGenericMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    tray.on('click', () => {
      windowController.toggle();
    });
    windowController.show();
  },

  _installLinuxWindowCloseHandler(windowController: WindowController) {
    windowController.window.on('close', (closeEvent) => {
      if (process.platform === 'linux' && !this._shouldQuit) {
        closeEvent.preventDefault();
        windowController.hide();
        return false;
      } else {
        return true;
      }
    });
  },
};

ApplicationMain.run();
