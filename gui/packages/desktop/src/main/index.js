// @flow

import fs from 'fs';
import log from 'electron-log';
import path from 'path';
import { execFile } from 'child_process';
import mkdirp from 'mkdirp';
import uuid from 'uuid';
import { app, screen, BrowserWindow, ipcMain, Tray, Menu, nativeImage } from 'electron';

import NotificationController from './notification-controller';
import WindowController from './window-controller';

import TrayIconController from './tray-icon-controller';
import type { TrayIconType } from './tray-icon-controller';

import IpcEventChannel from '../shared/ipc-event-channel';

import { DaemonRpc, ConnectionObserver, SubscriptionListener } from './daemon-rpc';
import type {
  AppVersionInfo,
  Location,
  RelayList,
  Settings,
  TunnelStateTransition,
} from './daemon-rpc';

import GuiSettings from './gui-settings';
import ReconnectionBackoff from './reconnection-backoff';
import { resolveBin } from './proc';

const RELAY_LIST_UPDATE_INTERVAL = 60 * 60 * 1000;
const VERSION_UPDATE_INTERVAL = 24 * 60 * 60 * 1000;

const DAEMON_RPC_PATH =
  process.platform === 'win32' ? '//./pipe/Mullvad VPN' : '/var/run/mullvad-vpn';

type AppQuitStage = 'unready' | 'initiated' | 'ready';

export type CurrentAppVersionInfo = {
  gui: string,
  daemon: string,
  isConsistent: boolean,
};

export type AppUpgradeInfo = {
  nextUpgrade: ?string,
  upToDate: boolean,
} & AppVersionInfo;

const ApplicationMain = {
  _notificationController: new NotificationController(),
  _windowController: (null: ?WindowController),
  _trayIconController: (null: ?TrayIconController),
  _ipcEventChannel: (null: ?IpcEventChannel),

  _daemonRpc: new DaemonRpc(),
  _reconnectBackoff: new ReconnectionBackoff(),
  _connectedToDaemon: false,

  _logFilePath: '',
  _oldLogFilePath: (null: ?string),
  _quitStage: ('unready': AppQuitStage),

  _tunnelState: ({ state: 'disconnected' }: TunnelStateTransition),
  _settings: ({
    accountToken: null,
    allowLan: false,
    autoConnect: false,
    blockWhenDisconnected: false,
    relaySettings: {
      normal: {
        location: 'any',
        tunnel: 'any',
      },
    },
    tunnelOptions: {
      enableIpv6: false,
      openvpn: {
        mssfix: null,
      },
      proxy: null,
    },
  }: Settings),
  _guiSettings: new GuiSettings(),
  _location: (null: ?Location),
  _lastDisconnectedLocation: (null: ?Location),

  _relays: ({ countries: [] }: RelayList),
  _relaysInterval: (null: ?IntervalID),

  _currentVersion: ({
    daemon: '',
    gui: '',
    isConsistent: true,
  }: CurrentAppVersionInfo),

  _upgradeVersion: ({
    currentIsSupported: true,
    latest: {
      latestStable: '',
      latest: '',
    },
    nextUpgrade: null,
    upToDate: true,
  }: AppUpgradeInfo),
  _latestVersionInterval: (null: ?IntervalID),

  run() {
    // Since electron's GPU blacklists are broken, GPU acceleration won't work on older distros
    if (process.platform === 'linux') {
      app.commandLine.appendSwitch('--disable-gpu');
    }

    this._overrideAppPaths();

    if (this._ensureSingleInstance()) {
      return;
    }

    this._initLogging();

    log.info(`Running version ${app.getVersion()}`);

    if (process.platform === 'win32') {
      app.setAppUserModelId('net.mullvad.vpn');
    }

    this._guiSettings.load();

    app.on('activate', () => this._onActivate());
    app.on('ready', () => this._onReady());
    app.on('window-all-closed', () => app.quit());
    app.on('before-quit', (event: Event) => this._onBeforeQuit(event));

    const connectionObserver = new ConnectionObserver(
      () => {
        this._onDaemonConnected();
      },
      (error) => {
        this._onDaemonDisconnected(error);
      },
    );

    this._daemonRpc.addConnectionObserver(connectionObserver);
    this._connectToDaemon();
  },

  _ensureSingleInstance() {
    if (app.requestSingleInstanceLock()) {
      app.on('second-instance', (_event, _commandLine, _workingDirectory) => {
        if (this._windowController) {
          this._windowController.show();
        }
      });
      return false;
    } else {
      app.quit();
      return true;
    }
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
    if (this._windowController) {
      this._windowController.show();
    }
  },

  async _onBeforeQuit(event: Event) {
    switch (this._quitStage) {
      case 'unready':
        // postpone the app shutdown
        event.preventDefault();

        this._quitStage = 'initiated';
        await this._prepareToQuit();

        // terminate the app
        this._quitStage = 'ready';
        app.quit();
        break;

      case 'initiated':
        // prevent immediate exit, the app will quit after running the shutdown routine
        event.preventDefault();
        return;

      case 'ready':
        // let the app quit freely at this point
        break;
    }
  },

  async _prepareToQuit() {
    if (this._connectedToDaemon) {
      try {
        await this._daemonRpc.disconnectTunnel();
        log.info('Disconnected the tunnel');
      } catch (e) {
        log.error(`Failed to disconnect the tunnel: ${e.message}`);
      }
    } else {
      log.info('Cannot close the tunnel because there is no active connection to daemon.');
    }
  },

  async _onReady() {
    const window = this._createWindow();
    const tray = this._createTray();

    const windowController = new WindowController(window, tray);
    const trayIconController = new TrayIconController(tray, 'unsecured');

    this._registerWindowListener(windowController);
    this._registerIpcListeners();
    this._setAppMenu();
    this._addContextMenu(window);

    this._windowController = windowController;
    this._trayIconController = trayIconController;
    this._ipcEventChannel = new IpcEventChannel(window.webContents);

    this._guiSettings.registerIpcHandlers(this._ipcEventChannel);

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

    if (this._shouldShowWindowOnStart() || process.env.NODE_ENV === 'development') {
      windowController.show();
    }

    window.loadFile(path.resolve(path.join(__dirname, '../renderer/index.html')));
  },

  async _onDaemonConnected() {
    this._connectedToDaemon = true;

    // subscribe to events
    try {
      await this._subscribeEvents();
    } catch (error) {
      log.error(`Failed to subscribe: ${error.message}`);

      return this._recoverFromBootstrapError(error);
    }

    // fetch the tunnel state
    try {
      this._setTunnelState(await this._daemonRpc.getState());
    } catch (error) {
      log.error(`Failed to fetch the tunnel state: ${error.message}`);

      return this._recoverFromBootstrapError(error);
    }

    // fetch settings
    try {
      this._setSettings(await this._daemonRpc.getSettings());
    } catch (error) {
      log.error(`Failed to fetch settings: ${error.message}`);

      return this._recoverFromBootstrapError(error);
    }

    // fetch relays
    try {
      this._setRelays(await this._daemonRpc.getRelayLocations());
    } catch (error) {
      log.error(`Failed to fetch relay locations: ${error.message}`);

      return this._recoverFromBootstrapError(error);
    }

    // fetch the daemon's version
    try {
      this._setDaemonVersion(await this._daemonRpc.getCurrentVersion());
    } catch (error) {
      log.error(`Failed to fetch the daemon's version: ${error.message}`);

      return this._recoverFromBootstrapError(error);
    }

    // fetch the latest version info in background
    this._fetchLatestVersion();

    // start periodic updates
    this._startRelaysPeriodicUpdates();
    this._startLatestVersionPeriodicUpdates();

    // notify user about inconsistent version
    if (
      process.env.NODE_ENV !== 'development' &&
      !this._shouldSuppressNotifications() &&
      !this._currentVersion.isConsistent
    ) {
      this._notificationController.notifyInconsistentVersion();
    }

    // reset the reconnect backoff when connection established.
    this._reconnectBackoff.reset();

    // notify renderer
    if (this._ipcEventChannel) {
      this._ipcEventChannel.daemonConnected.notify();
    }
  },

  _onDaemonDisconnected(error: ?Error) {
    // make sure we were connected before to distinguish between a failed attempt to reconnect and
    // connection loss.
    const wasConnected = this._connectedToDaemon;

    if (wasConnected) {
      this._connectedToDaemon = false;

      // stop periodic updates
      this._stopRelaysPeriodicUpdates();
      this._stopLatestVersionPeriodicUpdates();

      // notify renderer process
      if (this._ipcEventChannel) {
        this._ipcEventChannel.daemonDisconnected.notify(error ? error.message : null);
      }
    }

    // recover connection on error
    if (error) {
      if (wasConnected) {
        log.error(`Lost connection to daemon: ${error.message}`);
      } else {
        log.error(`Failed to connect to daemon: ${error.message}`);
      }

      this._reconnectToDaemon();
    } else {
      log.info('Disconnected from the daemon');
    }
  },

  _connectToDaemon() {
    this._daemonRpc.connect({ path: DAEMON_RPC_PATH });
  },

  _reconnectToDaemon() {
    this._reconnectBackoff.attempt(() => {
      this._connectToDaemon();
    });
  },

  _recoverFromBootstrapError(_error: ?Error) {
    // Attempt to reconnect to daemon if the program fails to fetch settings, tunnel state or
    // subscribe for RPC events.
    this._daemonRpc.disconnect();

    this._reconnectToDaemon();
  },

  async _subscribeEvents(): Promise<void> {
    const stateListener = new SubscriptionListener(
      (newState: TunnelStateTransition) => {
        this._setTunnelState(newState);
      },
      (error: Error) => {
        log.error(`Cannot deserialize the new state: ${error.message}`);
      },
    );

    const settingsListener = new SubscriptionListener(
      (newSettings: Settings) => {
        this._setSettings(newSettings);
      },
      (error: Error) => {
        log.error(`Cannot deserialize the new settings: ${error.message}`);
      },
    );

    await Promise.all([
      this._daemonRpc.subscribeStateListener(stateListener),
      this._daemonRpc.subscribeSettingsListener(settingsListener),
    ]);
  },

  _setTunnelState(newState: TunnelStateTransition) {
    this._tunnelState = newState;
    this._updateTrayIcon(newState, this._settings.blockWhenDisconnected);
    this._updateLocation();

    if (!this._shouldSuppressNotifications()) {
      this._notificationController.notifyTunnelState(newState);
    }

    if (this._ipcEventChannel) {
      this._ipcEventChannel.tunnelState.notify(newState);
    }
  },

  _setSettings(newSettings: Settings) {
    this._settings = newSettings;
    this._updateTrayIcon(this._tunnelState, newSettings.blockWhenDisconnected);

    if (this._ipcEventChannel) {
      this._ipcEventChannel.settings.notify(newSettings);
    }
  },

  _setLocation(newLocation: Location) {
    this._location = newLocation;

    if (this._ipcEventChannel) {
      this._ipcEventChannel.location.notify(newLocation);
    }
  },

  _setRelays(newRelayList: RelayList) {
    this._relays = newRelayList;

    if (this._ipcEventChannel) {
      this._ipcEventChannel.relays.notify(newRelayList);
    }
  },

  _startRelaysPeriodicUpdates() {
    log.debug('Start relays periodic updates');

    const handler = async () => {
      try {
        this._setRelays(await this._daemonRpc.getRelayLocations());
      } catch (error) {
        log.error(`Failed to fetch relay locations: ${error.message}`);
      }
    };

    this._relaysInterval = setInterval(handler, RELAY_LIST_UPDATE_INTERVAL);
  },

  _stopRelaysPeriodicUpdates() {
    if (this._relaysInterval) {
      clearInterval(this._relaysInterval);
      this._relaysInterval = null;

      log.debug('Stop relays periodic updates');
    }
  },

  _setDaemonVersion(daemonVersion: string) {
    const guiVersion = app.getVersion().replace('.0', '');
    const versionInfo = {
      daemon: daemonVersion,
      gui: guiVersion,
      isConsistent: daemonVersion === guiVersion,
    };

    this._currentVersion = versionInfo;

    // notify renderer
    if (this._ipcEventChannel) {
      this._ipcEventChannel.currentVersion.notify(versionInfo);
    }
  },

  _setLatestVersion(latestVersionInfo: AppVersionInfo) {
    function isBeta(version: string) {
      return version.includes('-');
    }

    function nextUpgrade(current: string, latest: string, latestStable: string): ?string {
      if (isBeta(current)) {
        return current === latest ? null : latest;
      } else {
        return current === latestStable ? null : latestStable;
      }
    }

    function checkIfLatest(current: string, latest: string, latestStable: string): boolean {
      // perhaps -beta?
      if (isBeta(current)) {
        return current === latest;
      } else {
        // must be stable
        return current === latestStable;
      }
    }

    const currentVersionInfo = this._currentVersion;
    const latestVersion = latestVersionInfo.latest.latest;
    const latestStableVersion = latestVersionInfo.latest.latestStable;

    // the reason why we rely on daemon version here is because daemon obtains the version info
    // based on its built-in version information
    const isUpToDate = checkIfLatest(currentVersionInfo.daemon, latestVersion, latestStableVersion);
    const upgradeVersion = nextUpgrade(
      currentVersionInfo.daemon,
      latestVersion,
      latestStableVersion,
    );

    const upgradeInfo = {
      ...latestVersionInfo,
      nextUpgrade: upgradeVersion,
      upToDate: isUpToDate,
    };

    this._upgradeVersion = upgradeInfo;

    // notify user to update the app if it became unsupported
    if (
      process.env.NODE_ENV !== 'development' &&
      !this._shouldSuppressNotifications() &&
      currentVersionInfo.isConsistent &&
      !latestVersionInfo.currentIsSupported &&
      upgradeVersion
    ) {
      this._notificationController.notifyUnsupportedVersion(upgradeVersion);
    }

    if (this._ipcEventChannel) {
      this._ipcEventChannel.upgradeVersion.notify(upgradeInfo);
    }
  },

  async _fetchLatestVersion() {
    try {
      this._setLatestVersion(await this._daemonRpc.getVersionInfo());
    } catch (error) {
      console.error(`Failed to request the version info: ${error.message}`);
    }
  },

  _startLatestVersionPeriodicUpdates() {
    const handler = () => {
      this._fetchLatestVersion();
    };
    this._latestVersionInterval = setInterval(handler, VERSION_UPDATE_INTERVAL);
  },

  _stopLatestVersionPeriodicUpdates() {
    if (this._latestVersionInterval) {
      clearInterval(this._latestVersionInterval);

      this._latestVersionInterval = null;
    }
  },

  _shouldSuppressNotifications() {
    return this._windowController && this._windowController.isVisible();
  },

  async _updateLocation() {
    const state = this._tunnelState.state;

    if (state === 'connected' || state === 'disconnected' || state === 'connecting') {
      try {
        // It may take some time to fetch the new user location.
        // So take the user to the last known location when disconnected.
        if (state === 'disconnected' && this._lastDisconnectedLocation) {
          this._setLocation(this._lastDisconnectedLocation);
        }

        // Fetch the new user location
        const location = await this._daemonRpc.getLocation();

        // Cache the user location
        // Note: hostname is only set for relay servers.
        if (location.hostname === null) {
          this._lastDisconnectedLocation = location;
        }

        // Broadcast the new location.
        // There is a chance that the location is not stale if the tunnel state before the location
        // request is the same as after receiving the response.
        if (this._tunnelState.state === state) {
          this._setLocation(location);
        }
      } catch (error) {
        log.error(`Failed to update the location: ${error.message}`);
      }
    }
  },

  _trayIconType(tunnelState: TunnelStateTransition, blockWhenDisconnected: boolean): TrayIconType {
    switch (tunnelState.state) {
      case 'connected':
        return 'secured';

      case 'connecting':
        return 'securing';

      case 'blocked':
        return 'securing';

      case 'disconnecting':
        switch (tunnelState.details) {
          case 'reconnect':
            return 'securing';

          case 'block':
            return 'securing';

          case 'nothing':
            // handle the same way as disconnected
            break;

          default:
            throw new Error(`Invalid after disconnect state: ${(tunnelState.details: empty)}`);
        }
      // fallthrough

      case 'disconnected':
        if (blockWhenDisconnected) {
          return 'securing';
        } else {
          return 'unsecured';
        }

      default:
        // unreachable, but can't prove it to flow because of the fallthrough
        throw new Error(`Invalid tunnel state: ${tunnelState.state}`);
    }
  },

  _updateTrayIcon(tunnelState: TunnelStateTransition, blockWhenDisconnected: boolean) {
    const type = this._trayIconType(tunnelState, blockWhenDisconnected);

    if (this._trayIconController) {
      this._trayIconController.animateToIcon(type);
    }
  },

  _registerWindowListener(windowController: WindowController) {
    windowController.window.on('show', () => {
      // cancel notifications when window appears
      this._notificationController.cancelPendingNotifications();

      windowController.send('window-shown');
    });
  },

  _registerIpcListeners() {
    ipcMain.on('daemon-rpc-call', async (event, id: string, method: string, payload: any) => {
      log.debug(`Got daemon-rpc-call: ${id} ${method}`);

      try {
        // $FlowFixMe: flow does not like index accessors.
        const result = await this._daemonRpc[method](payload);

        log.debug(`Send daemon-rpc-reply-${id} ${method} with success`);
        event.sender.send(`daemon-rpc-reply-${id}`, result);
      } catch (error) {
        log.debug(`Send daemon-rpc-reply-${id} ${method} with error: ${error.message}`);
        event.sender.send(`daemon-rpc-reply-${id}`, undefined, {
          className: error.constructor.name || '',
          data: {
            message: error.message,
            ...JSON.parse(JSON.stringify(error)),
          },
        });
      }
    });

    IpcEventChannel.state.serve(() => {
      return {
        isConnected: this._connectedToDaemon,
        tunnelState: this._tunnelState,
        settings: this._settings,
        location: this._location,
        relays: this._relays,
        currentVersion: this._currentVersion,
        upgradeVersion: this._upgradeVersion,
        guiSettings: this._guiSettings.state,
      };
    });

    ipcMain.on('show-window', () => {
      const windowController = this._windowController;
      if (windowController) {
        windowController.show();
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

    // disable double click on tray icon since it causes weird delay
    tray.setIgnoreDoubleClickEvents(true);

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
  },

  _installLinuxWindowCloseHandler(windowController: WindowController) {
    windowController.window.on('close', (closeEvent: Event) => {
      if (process.platform === 'linux' && this._quitStage !== 'ready') {
        closeEvent.preventDefault();
        windowController.hide();
      }
    });
  },

  _shouldShowWindowOnStart(): boolean {
    switch (process.platform) {
      case 'win32':
        return false;
      case 'darwin':
        return false;
      case 'linux':
        return !this._guiSettings.startMinimized;
      default:
        return true;
    }
  },
};

ApplicationMain.run();
