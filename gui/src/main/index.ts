import { exec, execFile } from 'child_process';
import { randomUUID } from 'crypto';
import { app, dialog, nativeTheme, session, shell, systemPreferences } from 'electron';
import fs from 'fs';
import * as path from 'path';
import util from 'util';

import config from '../config.json';
import { hasExpired } from '../shared/account-expiry';
import { IWindowsApplication } from '../shared/application-types';
import BridgeSettingsBuilder from '../shared/bridge-settings-builder';
import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  DaemonEvent,
  DeviceEvent,
  DeviceState,
  IAccountData,
  IDeviceRemoval,
  IDnsOptions,
  IRelayList,
  ISettings,
  liftConstraint,
  ObfuscationType,
  Ownership,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import { SYSTEM_PREFERRED_LOCALE_KEY } from '../shared/gui-settings-state';
import { ITranslations, MacOsScrollbarVisibility } from '../shared/ipc-schema';
import { IChangelog, IHistoryObject, ScrollPositions } from '../shared/ipc-types';
import log, { ConsoleOutput, Logger } from '../shared/logging';
import { LogLevel } from '../shared/logging-types';
import {
  AccountExpiredNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  SystemNotification,
} from '../shared/notifications/notification';
import { Scheduler } from '../shared/scheduler';
import AccountDataCache from './account-data-cache';
import { getOpenAtLogin, setOpenAtLogin } from './autostart';
import { readChangelog } from './changelog';
import { ConnectionObserver, DaemonRpc, SubscriptionListener } from './daemon-rpc';
import { InvalidAccountError } from './errors';
import Expectation from './expectation';
import GuiSettings from './gui-settings';
import { IpcMainEventChannel } from './ipc-event-channel';
import { findIconPath } from './linux-desktop-entry';
import { loadTranslations } from './load-translations';
import {
  backupLogFile,
  cleanUpLogDirectory,
  createLoggingDirectory,
  FileOutput,
  getMainLogPath,
  getRendererLogPath,
  IpcInput,
  OLD_LOG_FILES,
} from './logging';
import NotificationController, { NotificationControllerDelegate } from './notification-controller';
import { resolveBin } from './proc';
import ReconnectionBackoff from './reconnection-backoff';
import UserInterface, { UserInterfaceDelegate } from './user-interface';
import Version, { VersionDelegate } from './version';

const execAsync = util.promisify(exec);

// Only import split tunneling library on correct OS.
const linuxSplitTunneling = process.platform === 'linux' && require('./linux-split-tunneling');
const windowsSplitTunneling = process.platform === 'win32' && require('./windows-split-tunneling');

enum CommandLineOptions {
  quitWithoutDisconnect = '--quit-without-disconnect',
  showChanges = '--show-changes',
  disableResetNavigation = '--disable-reset-navigation', // development only
}

export enum AppQuitStage {
  unready,
  initiated,
  ready,
}

const ALLOWED_PERMISSIONS = ['clipboard-sanitized-write'];

const SANDBOX_DISABLED = app.commandLine.hasSwitch('no-sandbox');
const QUIT_WITHOUT_DISCONNECT = process.argv.includes(CommandLineOptions.quitWithoutDisconnect);
const FORCE_SHOW_CHANGES = process.argv.includes(CommandLineOptions.showChanges);
const NAVIGATION_RESET_DISABLED = process.argv.includes(CommandLineOptions.disableResetNavigation);
const UPDATE_NOTIFICATION_DISABLED = process.env.MULLVAD_DISABLE_UPDATE_NOTIFICATION === '1';

class ApplicationMain
  implements NotificationControllerDelegate, UserInterfaceDelegate, VersionDelegate {
  private notificationController = new NotificationController(this);
  private userInterface?: UserInterface;

  private version = new Version(this, UPDATE_NOTIFICATION_DISABLED);

  // True while file pickers are displayed which is used to decide if the Browser window should be
  // hidden when losing focus.
  private browsingFiles = false;

  private daemonRpc = new DaemonRpc();
  private daemonEventListener?: SubscriptionListener<DaemonEvent>;
  private reconnectBackoff = new ReconnectionBackoff();
  private beforeFirstDaemonConnection = true;
  private connectedToDaemon = false;
  private isPerformingPostUpgrade = false;
  private quitStage = AppQuitStage.unready;

  private accountData?: IAccountData = undefined;
  private accountHistory?: AccountToken = undefined;

  // The current tunnel state
  private tunnelState: TunnelState = { state: 'disconnected' };
  // When pressing connect/disconnect/reconnect the app assumes what the next state will be before
  // it get's the new state from the daemon. The latest state from the daemon is saved as fallback
  // if the assumed state isn't reached.
  private tunnelStateFallback?: TunnelState;
  // Scheduler for discarding the assumed next state.
  private tunnelStateFallbackScheduler = new Scheduler();

  private settings: ISettings = {
    allowLan: false,
    autoConnect: false,
    blockWhenDisconnected: false,
    showBetaReleases: false,
    splitTunnel: {
      enableExclusions: false,
      appsList: [],
    },
    relaySettings: {
      normal: {
        location: 'any',
        tunnelProtocol: 'any',
        providers: [],
        ownership: Ownership.any,
        openvpnConstraints: {
          port: 'any',
          protocol: 'any',
        },
        wireguardConstraints: {
          port: 'any',
          ipVersion: 'any',
          useMultihop: false,
          entryLocation: 'any',
        },
      },
    },
    bridgeSettings: {
      normal: {
        location: 'any',
        providers: [],
        ownership: Ownership.any,
      },
    },
    bridgeState: 'auto',
    tunnelOptions: {
      generic: {
        enableIpv6: false,
      },
      openvpn: {
        mssfix: undefined,
      },
      wireguard: {
        mtu: undefined,
      },
      dns: {
        state: 'default',
        defaultOptions: {
          blockAds: false,
          blockTrackers: false,
          blockMalware: false,
          blockAdultContent: false,
          blockGambling: false,
        },
        customOptions: {
          addresses: [],
        },
      },
    },
    obfuscationSettings: {
      selectedObfuscation: ObfuscationType.auto,
      udp2tcpSettings: {
        port: 'any',
      },
    },
  };
  private deviceState?: DeviceState;
  private guiSettings = new GuiSettings();
  private tunnelStateExpectation?: Expectation;

  private relays: IRelayList = { countries: [] };

  // The UI locale which is set once from onReady handler
  private locale = 'en';

  private accountExpiryNotificationScheduler = new Scheduler();

  private accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this.daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      this.accountData = accountData;

      IpcMainEventChannel.account.notify?.(this.accountData);

      this.handleAccountExpiry();
    },
  );

  private rendererLog?: Logger;
  private translations: ITranslations = { locale: this.locale };

  private windowsSplitTunnelingApplications?: IWindowsApplication[];

  private macOsScrollbarVisibility?: MacOsScrollbarVisibility;

  private stayConnectedOnQuit = false;

  private changelog?: IChangelog;

  private navigationHistory?: IHistoryObject;
  private scrollPositions: ScrollPositions = {};

  public run() {
    // Remove window animations to combat window flickering when opening window. Can be removed when
    // this issue has been resolved: https://github.com/electron/electron/issues/12130
    if (process.platform === 'win32') {
      app.commandLine.appendSwitch('wm-window-animations-disabled');
    }

    // Display correct colors regardless of monitor color profile.
    app.commandLine.appendSwitch('force-color-profile', 'srgb');

    this.overrideAppPaths();

    // This ensures that only a single instance is running at the same time, but also exits if
    // there's no already running instance when the quit without disconnect flag is supplied.
    if (!app.requestSingleInstanceLock() || QUIT_WITHOUT_DISCONNECT) {
      this.quitWithoutDisconnect();
      return;
    }

    this.addSecondInstanceEventHandler();

    this.initLogging();

    log.verbose(`Chromium sandbox is ${SANDBOX_DISABLED ? 'disabled' : 'enabled'}`);
    if (!SANDBOX_DISABLED) {
      app.enableSandbox();
    }

    log.info(`Running version ${this.version.currentVersion.gui}`);

    if (process.platform === 'win32') {
      app.setAppUserModelId('net.mullvad.vpn');
    }

    // While running in development the watch script triggers a reload of the renderer by sending
    // the signal `SIGUSR2`.
    if (process.env.NODE_ENV === 'development') {
      process.on('SIGUSR2', () => {
        this.userInterface?.reloadWindow();
      });
    }

    this.guiSettings.load();
    this.changelog = readChangelog();

    app.on('render-process-gone', (_event, _webContents, details) => {
      log.error(
        `Render process exited with exit code ${details.exitCode} due to ${details.reason}`,
      );
      this.quitWithoutDisconnect();
    });
    app.on('child-process-gone', (_event, details) => {
      log.error(
        `Child process of type ${details.type} exited with exit code ${details.exitCode} due to ${details.reason}`,
      );
    });

    app.on('activate', this.onActivate);
    app.on('ready', this.onReady);
    app.on('window-all-closed', () => app.quit());
    app.on('before-quit', this.onBeforeQuit);
    app.on('quit', this.onQuit);
  }

  private addSecondInstanceEventHandler() {
    app.on('second-instance', (_event, argv, _workingDirectory) => {
      if (argv.includes(CommandLineOptions.quitWithoutDisconnect)) {
        // Quit if another instance is started with the quit without disconnect flag.
        this.quitWithoutDisconnect();
      } else {
        // If no action was provided to the new instance the window is opened.
        this.userInterface?.showWindow();
      }
    });
  }

  private overrideAppPaths() {
    // This ensures that on Windows the %LOCALAPPDATA% directory is used instead of the %ADDDATA%
    // directory that has roaming contents
    if (process.platform === 'win32') {
      const appDataDir = process.env.LOCALAPPDATA;
      if (appDataDir) {
        const userDataDir = path.join(appDataDir, app.name);
        const logDir = path.join(userDataDir, 'logs');
        // In Electron 16, the `appData` directory must be created explicitly or an error is
        // thrown when creating the singleton lock file.
        fs.mkdirSync(logDir, { recursive: true });
        app.setPath('appData', appDataDir);
        app.setPath('userData', userDataDir);
        app.setPath('logs', logDir);
      } else {
        throw new Error('Missing %LOCALAPPDATA% environment variable');
      }
    } else if (process.platform === 'linux') {
      const userDataDir = app.getPath('userData');
      const logDir = path.join(userDataDir, 'logs');
      fs.mkdirSync(logDir, { recursive: true });
      app.setPath('logs', logDir);
    }
  }

  private initLogging() {
    const mainLogPath = getMainLogPath();
    const rendererLogPath = getRendererLogPath();

    if (process.env.NODE_ENV !== 'development') {
      this.rendererLog = new Logger();
      this.rendererLog.addInput(new IpcInput());

      try {
        createLoggingDirectory();
        cleanUpLogDirectory(OLD_LOG_FILES);

        backupLogFile(mainLogPath);
        backupLogFile(rendererLogPath);

        log.addOutput(new FileOutput(LogLevel.verbose, mainLogPath));
        this.rendererLog.addOutput(new FileOutput(LogLevel.verbose, rendererLogPath));
      } catch (e) {
        const error = e as Error;
        console.error('Failed to initialize logging:', error);
      }
    }

    log.addOutput(new ConsoleOutput(LogLevel.debug));
  }

  private onActivate = () => this.userInterface?.showWindow();

  private quitWithoutDisconnect() {
    this.stayConnectedOnQuit = true;
    app.quit();
  }

  // This is a last try to disconnect and quit gracefully if the app quits without having received
  // the before-quit event.
  private onQuit = async () => {
    if (this.quitStage !== AppQuitStage.ready) {
      await this.prepareToQuit();
    }
  };

  private onBeforeQuit = async (event: Electron.Event) => {
    switch (this.quitStage) {
      case AppQuitStage.unready:
        // postpone the app shutdown
        event.preventDefault();

        this.quitStage = AppQuitStage.initiated;

        log.info('Quit initiated');

        await this.prepareToQuit();

        // terminate the app
        this.quitStage = AppQuitStage.ready;
        app.quit();
        break;

      case AppQuitStage.initiated:
        // prevent immediate exit, the app will quit after running the shutdown routine
        event.preventDefault();
        return;

      case AppQuitStage.ready:
        // let the app quit freely at this point
        break;
    }
  };

  private async prepareToQuit() {
    if (this.stayConnectedOnQuit) {
      log.info('Not disconnecting tunnel on quit');
    } else {
      if (this.connectedToDaemon) {
        try {
          await this.daemonRpc.disconnectTunnel();
          log.info('Disconnected the tunnel');
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to disconnect the tunnel: ${error.message}`);
        }
      } else {
        log.info('Cannot close the tunnel because there is no active connection to daemon.');
      }
    }

    // Unsubscribe the event handler
    try {
      if (this.daemonEventListener) {
        this.daemonRpc.unsubscribeDaemonEventListener(this.daemonEventListener);

        log.info('Unsubscribed from the daemon events');
      }
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to unsubscribe from daemon events: ${error.message}`);
    }

    // The window is not closable on macOS to be able to hide the titlebar and workaround
    // a shadow bug rendered above the invisible title bar. This also prevents the window from
    // closing normally, even programmatically. Therefore re-enable the close button just before
    // quitting the app.
    // Github issue: https://github.com/electron/electron/issues/15008
    if (process.platform === 'darwin') {
      this.userInterface?.setWindowClosable(true);
    }

    if (this.connectedToDaemon) {
      this.daemonRpc.disconnect();
    }

    for (const logger of [log, this.rendererLog]) {
      try {
        logger?.dispose();
      } catch (e) {
        const error = e as Error;
        console.error('Failed to dispose logger:', error);
      }
    }

    this.userInterface?.dispose();
  }

  private detectLocale(): string {
    const preferredLocale = this.guiSettings.preferredLocale;
    if (preferredLocale === SYSTEM_PREFERRED_LOCALE_KEY) {
      return app.getLocale();
    } else {
      return preferredLocale;
    }
  }

  private onReady = async () => {
    // Disable built-in DNS resolver.
    app.configureHostResolver({
      enableBuiltInResolver: false,
      secureDnsMode: 'off',
      secureDnsServers: [],
    });

    // There's no option that prevents Electron from fetching spellcheck dictionaries from
    // Chromium's CDN and passing a non-resolving URL is the only known way to prevent it from
    // fetching.  https://github.com/electron/electron/issues/22995
    session.defaultSession.setSpellCheckerDictionaryDownloadURL('https://00.00/');

    // Blocks scripts in the renderer process from asking for any permission.
    this.blockPermissionRequests();
    // Blocks any http(s) and file requests that aren't supposed to happen.
    this.blockRequests();
    // Blocks navigation and window.open since it's not needed.
    this.blockNavigationAndWindowOpen();

    this.updateCurrentLocale();

    this.daemonRpc.addConnectionObserver(
      new ConnectionObserver(this.onDaemonConnected, this.onDaemonDisconnected),
    );
    this.connectToDaemon();

    if (process.platform === 'darwin') {
      await this.updateMacOsScrollbarVisibility();
      systemPreferences.subscribeNotification('AppleShowScrollBarsSettingChanged', async () => {
        await this.updateMacOsScrollbarVisibility();
      });
    }

    this.userInterface = new UserInterface(this, SANDBOX_DISABLED, NAVIGATION_RESET_DISABLED);

    this.tunnelStateExpectation = new Expectation(async () => {
      this.userInterface?.createTrayIconController(
        this.tunnelState,
        this.settings.blockWhenDisconnected,
        this.guiSettings.monochromaticIcon,
      );
      await this.userInterface?.updateTrayTheme();

      this.userInterface?.setTrayContextMenu();
      this.userInterface?.setTrayTooltip();

      if (process.platform === 'win32') {
        nativeTheme.on('updated', async () => {
          if (this.guiSettings.monochromaticIcon) {
            await this.userInterface?.updateTrayTheme();
          }
        });
      }
    });

    this.registerIpcListeners();

    this.guiSettings.onChange = async (newState, oldState) => {
      if (oldState.monochromaticIcon !== newState.monochromaticIcon) {
        await this.userInterface?.setUseMonochromaticTrayIcon(newState.monochromaticIcon);
      }

      if (newState.autoConnect !== oldState.autoConnect) {
        this.updateDaemonsAutoConnect();
      }

      IpcMainEventChannel.guiSettings.notify?.(newState);
    };

    if (this.shouldShowWindowOnStart() || process.env.NODE_ENV === 'development') {
      this.userInterface.showWindow();
    }

    if (process.platform === 'linux') {
      const icon = await findIconPath('mullvad-vpn');
      if (icon) {
        this.userInterface.setWindowIcon(icon);
      }
    }

    await this.userInterface.initializeWindow();
  };

  private onDaemonConnected = async () => {
    const firstDaemonConnection = this.beforeFirstDaemonConnection;
    this.beforeFirstDaemonConnection = false;
    this.connectedToDaemon = true;

    log.info('Connected to the daemon');

    // subscribe to events
    try {
      this.daemonEventListener = this.subscribeEvents();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to subscribe: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    if (firstDaemonConnection) {
      // check if daemon is performing post upgrade tasks the first time it's connected to
      try {
        await this.performPostUpgradeCheck();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to check if daemon is performing post upgrade tasks: ${error.message}`);

        return this.handleBootstrapError(error);
      }
    }

    // fetch account history
    try {
      this.setAccountHistory(await this.daemonRpc.getAccountHistory());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the account history: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch the tunnel state
    try {
      this.setTunnelState(await this.daemonRpc.getState());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the tunnel state: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch device
    try {
      const deviceState = await this.daemonRpc.getDevice();
      this.handleDeviceEvent({ type: deviceState.type, deviceState } as DeviceEvent);
      if (deviceState.type === 'logged in') {
        void this.daemonRpc
          .updateDevice()
          .catch((error: Error) => log.warn(`Failed to update device info: ${error.message}`));
      }
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch device: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch settings
    try {
      this.setSettings(await this.daemonRpc.getSettings());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch settings: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    if (this.tunnelStateExpectation) {
      this.tunnelStateExpectation.fulfill();
    }

    // fetch relays
    try {
      this.setRelays(
        await this.daemonRpc.getRelayLocations(),
        this.settings.relaySettings,
        this.settings.bridgeState,
      );
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch relay locations: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch the daemon's version
    try {
      this.version.setDaemonVersion(await this.daemonRpc.getCurrentVersion());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the daemon's version: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch the latest version info in background
    if (!UPDATE_NOTIFICATION_DISABLED) {
      void this.version.fetchLatestVersion();
    }

    // reset the reconnect backoff when connection established.
    this.reconnectBackoff.reset();

    // notify renderer, this.connectedToDaemon could have changed if the daemon disconnected again
    // before this if-statement is reached.
    if (this.connectedToDaemon) {
      IpcMainEventChannel.daemon.notifyConnected?.();
    }

    if (firstDaemonConnection) {
      void this.autoConnect();
    }

    // show window when account is not set
    if (!this.isLoggedIn()) {
      this.userInterface?.showWindow();
    }
  };

  private onDaemonDisconnected = (error?: Error) => {
    if (this.daemonEventListener) {
      this.daemonRpc.unsubscribeDaemonEventListener(this.daemonEventListener);
    }
    // make sure we were connected before to distinguish between a failed attempt to reconnect and
    // connection loss.
    const wasConnected = this.connectedToDaemon;

    // Reset the daemon event listener since it's going to be invalidated on disconnect
    this.daemonEventListener = undefined;

    this.tunnelStateFallback = undefined;

    if (wasConnected) {
      this.connectedToDaemon = false;

      // update the tray icon to indicate that the computer is not secure anymore
      this.userInterface?.updateTrayIcon({ state: 'disconnected' }, false);
      this.userInterface?.setTrayContextMenu();
      this.userInterface?.setTrayTooltip();

      // notify renderer process
      IpcMainEventChannel.daemon.notifyDisconnected?.();
    }

    // recover connection on error
    if (error) {
      if (wasConnected) {
        log.error(`Lost connection to daemon: ${error.message}`);
      } else {
        log.error(`Failed to connect to daemon: ${error.message}`);
      }
    } else {
      log.info('Disconnected from the daemon');
    }
  };

  private connectToDaemon() {
    void this.daemonRpc
      .connect()
      .catch((error) => log.error(`Unable to connect to daemon: ${error.message}`));
  }

  private handleBootstrapError(_error?: Error) {
    // Unsubscribe from daemon events when encountering errors during initial data retrieval.
    if (this.daemonEventListener) {
      this.daemonRpc.unsubscribeDaemonEventListener(this.daemonEventListener);
    }
  }

  private subscribeEvents(): SubscriptionListener<DaemonEvent> {
    const daemonEventListener = new SubscriptionListener(
      (daemonEvent: DaemonEvent) => {
        if ('tunnelState' in daemonEvent) {
          this.setTunnelState(daemonEvent.tunnelState);
        } else if ('settings' in daemonEvent) {
          this.setSettings(daemonEvent.settings);
        } else if ('relayList' in daemonEvent) {
          this.setRelays(
            daemonEvent.relayList,
            this.settings.relaySettings,
            this.settings.bridgeState,
          );
        } else if ('appVersionInfo' in daemonEvent) {
          this.version.setLatestVersion(daemonEvent.appVersionInfo);
        } else if ('device' in daemonEvent) {
          this.handleDeviceEvent(daemonEvent.device);
        } else if ('deviceRemoval' in daemonEvent) {
          IpcMainEventChannel.account.notifyDevices?.(daemonEvent.deviceRemoval);
        }
      },
      (error: Error) => {
        log.error(`Cannot deserialize the daemon event: ${error.message}`);
      },
    );

    this.daemonRpc.subscribeDaemonEventListener(daemonEventListener);

    return daemonEventListener;
  }

  private async performPostUpgradeCheck(): Promise<void> {
    const oldValue = this.isPerformingPostUpgrade;
    this.isPerformingPostUpgrade = await this.daemonRpc.isPerformingPostUpgrade();
    if (this.isPerformingPostUpgrade !== oldValue) {
      IpcMainEventChannel.daemon.notifyIsPerformingPostUpgrade?.(this.isPerformingPostUpgrade);
    }
  }

  private setAccountHistory(accountHistory?: AccountToken) {
    this.accountHistory = accountHistory;

    IpcMainEventChannel.accountHistory.notify?.(accountHistory);
  }

  // This function sets a new tunnel state as an assumed next state and saves the current state as
  // fallback. The fallback is used if the assumed next state isn't reached.
  private setOptimisticTunnelState(state: 'connecting' | 'disconnecting') {
    this.tunnelStateFallback = this.tunnelState;

    this.setTunnelStateImpl(
      state === 'disconnecting' ? { state, details: 'nothing' as const } : { state },
    );

    this.tunnelStateFallbackScheduler.schedule(() => {
      if (this.tunnelStateFallback) {
        this.setTunnelStateImpl(this.tunnelStateFallback);
        this.tunnelStateFallback = undefined;
      }
    }, 3000);
  }

  private setTunnelState(newState: TunnelState) {
    // If there's a fallback state set then the app is in an assumed next state and need to check
    // if it's now reached or if the current state should be ignored and set as the fallback state.
    if (this.tunnelStateFallback) {
      if (this.tunnelState.state === newState.state || newState.state === 'error') {
        this.tunnelStateFallbackScheduler.cancel();
        this.tunnelStateFallback = undefined;
      } else {
        this.tunnelStateFallback = newState;
        return;
      }
    }

    if (newState.state === 'disconnecting' && newState.details === 'reconnect') {
      // When reconnecting there's no need of showing the disconnecting state. This switches to the
      // connecting state immediately.
      this.setOptimisticTunnelState('connecting');
      this.tunnelStateFallback = newState;
    } else {
      this.setTunnelStateImpl(newState);
    }
  }

  private setTunnelStateImpl(newState: TunnelState) {
    this.tunnelState = newState;
    this.userInterface?.updateTrayIcon(newState, this.settings.blockWhenDisconnected);

    this.userInterface?.setTrayContextMenu();
    this.userInterface?.setTrayTooltip();

    this.notificationController.notifyTunnelState(
      newState,
      this.settings.blockWhenDisconnected,
      this.settings.splitTunnel.enableExclusions && this.settings.splitTunnel.appsList.length > 0,
      this.accountData?.expiry,
    );

    IpcMainEventChannel.tunnel.notify?.(newState);

    if (this.accountData) {
      this.detectStaleAccountExpiry(newState, new Date(this.accountData.expiry));
    }
  }

  private setSettings(newSettings: ISettings) {
    const oldSettings = this.settings;
    this.settings = newSettings;

    this.userInterface?.updateTrayIcon(this.tunnelState, newSettings.blockWhenDisconnected);

    if (oldSettings.showBetaReleases !== newSettings.showBetaReleases) {
      this.version.setLatestVersion(this.version.upgradeVersion);
    }

    IpcMainEventChannel.settings.notify?.(newSettings);

    if (windowsSplitTunneling) {
      void this.updateSplitTunnelingApplications(newSettings.splitTunnel.appsList);
    }

    // since settings can have the relay constraints changed, the relay
    // list should also be updated
    this.setRelays(this.relays, newSettings.relaySettings, newSettings.bridgeState);
  }

  private async updateSplitTunnelingApplications(appList: string[]): Promise<void> {
    const { applications } = await windowsSplitTunneling.getApplications({
      applicationPaths: appList,
    });
    this.windowsSplitTunnelingApplications = applications;

    IpcMainEventChannel.windowsSplitTunneling.notify?.(applications);
  }

  private setRelays(
    newRelayList: IRelayList,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
  ) {
    this.relays = newRelayList;

    const filteredRelays = this.processRelaysForPresentation(newRelayList, relaySettings);
    const filteredBridges = this.processBridgesForPresentation(newRelayList, bridgeState);

    IpcMainEventChannel.relays.notify?.({ relays: filteredRelays, bridges: filteredBridges });
  }

  private processRelaysForPresentation(
    relayList: IRelayList,
    relaySettings: RelaySettings,
  ): IRelayList {
    const tunnelProtocol =
      'normal' in relaySettings ? liftConstraint(relaySettings.normal.tunnelProtocol) : undefined;

    const filteredCountries = relayList.countries
      .map((country) => ({
        ...country,
        cities: country.cities
          .map((city) => ({
            ...city,
            relays: city.relays.filter((relay) => {
              if (relay.endpointType != 'bridge') {
                switch (tunnelProtocol) {
                  case 'openvpn':
                    return relay.endpointType == 'openvpn';

                  case 'wireguard':
                    return relay.endpointType == 'wireguard';

                  case 'any': {
                    const useMultihop =
                      'normal' in relaySettings &&
                      relaySettings.normal.wireguardConstraints.useMultihop;
                    return !useMultihop || relay.endpointType == 'wireguard';
                  }
                  default:
                    return false;
                }
              } else {
                return false;
              }
            }),
          }))
          .filter((city) => city.relays.length > 0),
      }))
      .filter((country) => country.cities.length > 0);

    return {
      countries: filteredCountries,
    };
  }

  private processBridgesForPresentation(
    relayList: IRelayList,
    bridgeState: BridgeState,
  ): IRelayList {
    if (bridgeState === 'on') {
      const filteredCountries = relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => relay.endpointType == 'bridge'),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0);

      return { countries: filteredCountries };
    } else {
      return { countries: [] };
    }
  }
  private handleDeviceEvent(deviceEvent: DeviceEvent) {
    this.deviceState = deviceEvent.deviceState;

    if (this.isPerformingPostUpgrade) {
      void this.performPostUpgradeCheck();
    }

    switch (deviceEvent.deviceState.type) {
      case 'logged in':
        this.accountDataCache.fetch(deviceEvent.deviceState.accountAndDevice.accountToken);
        break;
      case 'logged out':
      case 'revoked':
        this.accountDataCache.invalidate();
        break;
    }

    void this.updateAccountHistory();
    this.userInterface?.setTrayContextMenu();

    IpcMainEventChannel.account.notifyDevice?.(deviceEvent);
  }

  private getAccountToken(): AccountToken | undefined {
    return this.deviceState?.type === 'logged in'
      ? this.deviceState.accountAndDevice.accountToken
      : undefined;
  }

  private registerIpcListeners() {
    IpcMainEventChannel.state.handleGet(() => ({
      isConnected: this.connectedToDaemon,
      autoStart: getOpenAtLogin(),
      accountData: this.accountData,
      accountHistory: this.accountHistory,
      tunnelState: this.tunnelState,
      settings: this.settings,
      isPerformingPostUpgrade: this.isPerformingPostUpgrade,
      deviceState: this.deviceState,
      relayListPair: {
        relays: this.processRelaysForPresentation(this.relays, this.settings.relaySettings),
        bridges: this.processBridgesForPresentation(this.relays, this.settings.bridgeState),
      },
      currentVersion: this.version.currentVersion,
      upgradeVersion: this.version.upgradeVersion,
      guiSettings: this.guiSettings.state,
      translations: this.translations,
      windowsSplitTunnelingApplications: this.windowsSplitTunnelingApplications,
      macOsScrollbarVisibility: this.macOsScrollbarVisibility,
      changelog: this.changelog ?? [],
      forceShowChanges: FORCE_SHOW_CHANGES,
      navigationHistory: this.navigationHistory,
      scrollPositions: this.scrollPositions,
    }));

    IpcMainEventChannel.settings.handleSetAllowLan((allowLan: boolean) =>
      this.daemonRpc.setAllowLan(allowLan),
    );
    IpcMainEventChannel.settings.handleSetShowBetaReleases((showBetaReleases: boolean) =>
      this.daemonRpc.setShowBetaReleases(showBetaReleases),
    );
    IpcMainEventChannel.settings.handleSetEnableIpv6((enableIpv6: boolean) =>
      this.daemonRpc.setEnableIpv6(enableIpv6),
    );
    IpcMainEventChannel.settings.handleSetBlockWhenDisconnected((blockWhenDisconnected: boolean) =>
      this.daemonRpc.setBlockWhenDisconnected(blockWhenDisconnected),
    );
    IpcMainEventChannel.settings.handleSetBridgeState(async (bridgeState: BridgeState) => {
      await this.daemonRpc.setBridgeState(bridgeState);

      // Reset bridge constraints to `any` when the state is set to auto or off
      if (bridgeState === 'auto' || bridgeState === 'off') {
        await this.daemonRpc.setBridgeSettings(new BridgeSettingsBuilder().location.any().build());
      }
    });
    IpcMainEventChannel.settings.handleSetOpenVpnMssfix((mssfix?: number) =>
      this.daemonRpc.setOpenVpnMssfix(mssfix),
    );
    IpcMainEventChannel.settings.handleSetWireguardMtu((mtu?: number) =>
      this.daemonRpc.setWireguardMtu(mtu),
    );
    IpcMainEventChannel.settings.handleUpdateRelaySettings((update: RelaySettingsUpdate) =>
      this.daemonRpc.updateRelaySettings(update),
    );
    IpcMainEventChannel.settings.handleUpdateBridgeSettings((bridgeSettings: BridgeSettings) => {
      return this.daemonRpc.setBridgeSettings(bridgeSettings);
    });
    IpcMainEventChannel.settings.handleSetDnsOptions((dns: IDnsOptions) => {
      return this.daemonRpc.setDnsOptions(dns);
    });
    IpcMainEventChannel.autoStart.handleSet((autoStart: boolean) => {
      return this.setAutoStart(autoStart);
    });
    IpcMainEventChannel.settings.handleSetObfuscationSettings((obfuscationSettings) => {
      return this.daemonRpc.setObfuscationSettings(obfuscationSettings);
    });

    IpcMainEventChannel.location.handleGet(() => this.daemonRpc.getLocation());

    IpcMainEventChannel.tunnel.handleConnect(this.connectTunnel);
    IpcMainEventChannel.tunnel.handleReconnect(this.reconnectTunnel);
    IpcMainEventChannel.tunnel.handleDisconnect(this.disconnectTunnel);

    IpcMainEventChannel.guiSettings.handleSetEnableSystemNotifications((flag: boolean) => {
      this.guiSettings.enableSystemNotifications = flag;
    });

    IpcMainEventChannel.guiSettings.handleSetAutoConnect((autoConnect: boolean) => {
      this.guiSettings.autoConnect = autoConnect;
    });

    IpcMainEventChannel.guiSettings.handleSetStartMinimized((startMinimized: boolean) => {
      this.guiSettings.startMinimized = startMinimized;
    });

    IpcMainEventChannel.guiSettings.handleSetMonochromaticIcon((monochromaticIcon: boolean) => {
      this.guiSettings.monochromaticIcon = monochromaticIcon;
    });

    IpcMainEventChannel.guiSettings.handleSetUnpinnedWindow((unpinnedWindow: boolean) => {
      void this.setUnpinnedWindow(unpinnedWindow);
    });

    IpcMainEventChannel.guiSettings.handleSetPreferredLocale((locale: string) => {
      this.guiSettings.preferredLocale = locale;
      this.updateCurrentLocale();
      return Promise.resolve(this.translations);
    });

    IpcMainEventChannel.account.handleCreate(() => this.createNewAccount());
    IpcMainEventChannel.account.handleLogin((token: AccountToken) => this.login(token));
    IpcMainEventChannel.account.handleLogout(() => this.logout());
    IpcMainEventChannel.account.handleGetWwwAuthToken(() => this.daemonRpc.getWwwAuthToken());
    IpcMainEventChannel.account.handleSubmitVoucher(async (voucherCode: string) => {
      const currentAccountToken = this.getAccountToken();
      const response = await this.daemonRpc.submitVoucher(voucherCode);

      if (currentAccountToken) {
        this.accountDataCache.handleVoucherResponse(currentAccountToken, response);
      }

      return response;
    });
    IpcMainEventChannel.account.handleUpdateData(() => this.updateAccountData());

    IpcMainEventChannel.account.handleGetDeviceState(async () => {
      try {
        await this.daemonRpc.updateDevice();
      } catch (e) {
        const error = e as Error;
        log.warn(`Failed to update device info: ${error.message}`);
      }
      return this.daemonRpc.getDevice();
    });
    IpcMainEventChannel.account.handleListDevices((accountToken: AccountToken) => {
      return this.daemonRpc.listDevices(accountToken);
    });
    IpcMainEventChannel.account.handleRemoveDevice((deviceRemoval: IDeviceRemoval) => {
      return this.daemonRpc.removeDevice(deviceRemoval);
    });

    IpcMainEventChannel.accountHistory.handleClear(async () => {
      await this.daemonRpc.clearAccountHistory();
      void this.updateAccountHistory();
    });

    IpcMainEventChannel.linuxSplitTunneling.handleGetApplications(() => {
      return linuxSplitTunneling.getApplications(this.locale);
    });
    IpcMainEventChannel.windowsSplitTunneling.handleGetApplications((updateCaches: boolean) => {
      return windowsSplitTunneling.getApplications({ updateCaches });
    });
    IpcMainEventChannel.linuxSplitTunneling.handleLaunchApplication((application) => {
      return linuxSplitTunneling.launchApplication(application);
    });

    IpcMainEventChannel.windowsSplitTunneling.handleSetState((enabled) => {
      return this.daemonRpc.setSplitTunnelingState(enabled);
    });
    IpcMainEventChannel.windowsSplitTunneling.handleAddApplication(async (application) => {
      // If the applications is a string (path) it's an application picked with the file picker
      // that we want to add to the list of additional applications.
      if (typeof application === 'string') {
        this.guiSettings.addBrowsedForSplitTunnelingApplications(application);
        const applicationPath = await windowsSplitTunneling.addApplicationPathToCache(application);
        await this.daemonRpc.addSplitTunnelingApplication(applicationPath);
      } else {
        await this.daemonRpc.addSplitTunnelingApplication(application.absolutepath);
      }
    });
    IpcMainEventChannel.windowsSplitTunneling.handleRemoveApplication((application) => {
      return this.daemonRpc.removeSplitTunnelingApplication(
        typeof application === 'string' ? application : application.absolutepath,
      );
    });
    IpcMainEventChannel.windowsSplitTunneling.handleForgetManuallyAddedApplication(
      (application) => {
        this.guiSettings.deleteBrowsedForSplitTunnelingApplications(application.absolutepath);
        return windowsSplitTunneling.removeApplicationFromCache(application);
      },
    );

    IpcMainEventChannel.problemReport.handleCollectLogs((toRedact) => {
      const id = randomUUID();
      const reportPath = this.getProblemReportPath(id);
      const executable = resolveBin('mullvad-problem-report');
      const args = ['collect', '--output', reportPath];
      if (toRedact) {
        args.push('--redact', toRedact);
      }

      return new Promise((resolve, reject) => {
        execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
          if (error) {
            log.error(
              `Failed to collect a problem report.
                Stdout: ${stdout.toString()}
                Stderr: ${stderr.toString()}`,
            );
            reject(error.message);
          } else {
            log.verbose(`Problem report was written to ${reportPath}`);
            resolve(id);
          }
        });
      });
    });

    IpcMainEventChannel.problemReport.handleSendReport(({ email, message, savedReportId }) => {
      const executable = resolveBin('mullvad-problem-report');
      const reportPath = this.getProblemReportPath(savedReportId);
      const args = ['send', '--email', email, '--message', message, '--report', reportPath];

      return new Promise((resolve, reject) => {
        execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
          if (error) {
            log.error(
              `Failed to send a problem report.
              Stdout: ${stdout.toString()}
              Stderr: ${stderr.toString()}`,
            );
            reject(error.message);
          } else {
            log.info('Problem report was sent.');
            resolve();
          }
        });
      });
    });

    IpcMainEventChannel.problemReport.handleViewLog((savedReportId) =>
      shell.openPath(this.getProblemReportPath(savedReportId)),
    );

    IpcMainEventChannel.app.handleQuit(() => app.quit());
    IpcMainEventChannel.app.handleOpenUrl(async (url) => {
      if (Object.values(config.links).find((link) => url.startsWith(link))) {
        await shell.openExternal(url);
      }
    });
    IpcMainEventChannel.app.handleShowOpenDialog(async (options) => {
      this.browsingFiles = true;
      const response = await dialog.showOpenDialog({
        defaultPath: app.getPath('home'),
        ...options,
      });
      this.browsingFiles = false;
      return response;
    });

    IpcMainEventChannel.currentVersion.handleDisplayedChangelog(() => {
      this.guiSettings.changelogDisplayedForVersion = this.version.currentVersion.gui;
    });

    IpcMainEventChannel.navigation.handleSetHistory((history) => {
      this.navigationHistory = history;
    });
    IpcMainEventChannel.navigation.handleSetScrollPositions((scrollPositions) => {
      this.scrollPositions = scrollPositions;
    });

    if (windowsSplitTunneling) {
      this.guiSettings.browsedForSplitTunnelingApplications.forEach(
        windowsSplitTunneling.addApplicationPathToCache,
      );
    }
  }

  private async createNewAccount(): Promise<string> {
    try {
      return await this.daemonRpc.createNewAccount();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to create account: ${error.message}`);
      throw error;
    }
  }

  private async login(accountToken: AccountToken): Promise<void> {
    try {
      await this.daemonRpc.loginAccount(accountToken);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to login: ${error.message}`);

      if (error instanceof InvalidAccountError) {
        throw Error(messages.gettext('Invalid account number'));
      } else {
        throw error;
      }
    }
  }

  private async autoConnect() {
    if (process.env.NODE_ENV === 'development') {
      log.info('Skip autoconnect in development');
    } else if (this.isLoggedIn() && (!this.accountData || !hasExpired(this.accountData.expiry))) {
      if (this.guiSettings.autoConnect) {
        try {
          log.info('Autoconnect the tunnel');

          await this.daemonRpc.connectTunnel();
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to autoconnect the tunnel: ${error.message}`);
        }
      } else {
        log.info('Skip autoconnect because GUI setting is disabled');
      }
    } else {
      log.info('Skip autoconnect because account token is not set');
    }
  }

  private async logout(): Promise<void> {
    try {
      await this.daemonRpc.logoutAccount();

      this.accountExpiryNotificationScheduler.cancel();
    } catch (e) {
      const error = e as Error;
      log.info(`Failed to logout: ${error.message}`);

      throw error;
    }
  }

  private detectStaleAccountExpiry(tunnelState: TunnelState, accountExpiry: Date) {
    const hasExpired = new Date() >= accountExpiry;

    // It's likely that the account expiry is stale if the daemon managed to establish the tunnel.
    if (tunnelState.state === 'connected' && hasExpired) {
      log.info('Detected the stale account expiry.');
      this.accountDataCache.invalidate();
    }
  }

  private handleAccountExpiry() {
    if (this.accountData) {
      const expiredNotification = new AccountExpiredNotificationProvider({
        accountExpiry: this.accountData.expiry,
        tunnelState: this.tunnelState,
      });
      const closeToExpiryNotification = new CloseToAccountExpiryNotificationProvider({
        accountExpiry: this.accountData.expiry,
        locale: this.locale,
      });

      if (expiredNotification.mayDisplay()) {
        this.accountExpiryNotificationScheduler.cancel();
        this.notificationController.notify(expiredNotification.getSystemNotification());
      } else if (
        !this.accountExpiryNotificationScheduler.isRunning &&
        closeToExpiryNotification.mayDisplay()
      ) {
        this.notificationController.notify(closeToExpiryNotification.getSystemNotification());

        const twelveHours = 12 * 60 * 60 * 1000;
        const remainingMilliseconds = new Date(this.accountData.expiry).getTime() - Date.now();
        const delay = Math.min(twelveHours, remainingMilliseconds);
        this.accountExpiryNotificationScheduler.schedule(() => this.handleAccountExpiry(), delay);
      }
    }
  }

  private async updateAccountHistory(): Promise<void> {
    try {
      this.setAccountHistory(await this.daemonRpc.getAccountHistory());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the account history: ${error.message}`);
    }
  }

  private updateDaemonsAutoConnect() {
    const daemonAutoConnect = this.guiSettings.autoConnect && getOpenAtLogin();
    if (daemonAutoConnect !== this.settings.autoConnect) {
      void this.daemonRpc.setAutoConnect(daemonAutoConnect);
    }
  }

  private async setAutoStart(autoStart: boolean): Promise<void> {
    try {
      await setOpenAtLogin(autoStart);

      IpcMainEventChannel.autoStart.notify?.(autoStart);

      this.updateDaemonsAutoConnect();
    } catch (e) {
      const error = e as Error;
      log.error(
        `Failed to update the autostart to ${autoStart.toString()}. ${error.message.toString()}`,
      );
    }
    return Promise.resolve();
  }

  private async setUnpinnedWindow(unpinnedWindow: boolean) {
    this.guiSettings.unpinnedWindow = unpinnedWindow;
    await this.userInterface?.recreateWindow();
  }

  private updateCurrentLocale() {
    this.locale = this.detectLocale();

    log.info(`Detected locale: ${this.locale}`);

    const messagesTranslations = loadTranslations(this.locale, messages);
    const relayLocationsTranslations = loadTranslations(this.locale, relayLocations);

    this.translations = {
      locale: this.locale,
      messages: messagesTranslations,
      relayLocations: relayLocationsTranslations,
    };

    this.userInterface?.setTrayContextMenu();
    this.userInterface?.setTrayTooltip();
  }

  private blockPermissionRequests() {
    session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => {
      callback(false);
    });
    session.defaultSession.setPermissionCheckHandler((_webContents, permission) =>
      ALLOWED_PERMISSIONS.includes(permission),
    );
  }

  // Since the app frontend never performs any network requests, all requests originating from the
  // renderer process are blocked to protect against the potential threat of malicious third party
  // dependencies. There are a few exceptions which are described further down.
  private blockRequests() {
    session.defaultSession.webRequest.onBeforeRequest((details, callback) => {
      if (this.allowFileAccess(details.url) || this.allowDevelopmentRequest(details.url)) {
        callback({});
      } else {
        log.error(`${details.method} request blocked: ${details.url}`);
        callback({ cancel: true });

        // Throw error in development to notify since this should never happen.
        if (process.env.NODE_ENV === 'development') {
          throw new Error('Web request blocked');
        }
      }
    });
  }

  private allowFileAccess(url: string): boolean {
    const buildDir = path.normalize(path.join(path.resolve(__dirname), '..', '..'));

    if (url.startsWith('file:')) {
      // Extract the path from the URL
      let filePath = decodeURI(new URL(url).pathname);
      if (process.platform === 'win32') {
        // Windows paths shouldn't start with a '/'
        filePath = filePath.replace(/^\//, '');
      }
      filePath = path.resolve(filePath);

      return !path.relative(buildDir, filePath).includes('..');
    } else {
      return false;
    }
  }

  private allowDevelopmentRequest(url: string): boolean {
    return (
      process.env.NODE_ENV === 'development' &&
      // Downloading of React and Redux developer tools.
      (url.startsWith('devtools://devtools/') ||
        url.startsWith('chrome-extension://') ||
        url.startsWith('https://clients2.google.com') ||
        url.startsWith('https://clients2.googleusercontent.com'))
    );
  }

  // Blocks navigation and window.open since it's not needed.
  private blockNavigationAndWindowOpen() {
    app.on('web-contents-created', (_event, contents) => {
      contents.on('will-navigate', (event) => event.preventDefault());
      contents.setWindowOpenHandler(() => ({ action: 'deny' }));
    });
  }

  private shouldShowWindowOnStart(): boolean {
    switch (process.platform) {
      case 'win32':
      case 'darwin':
      case 'linux':
        return this.guiSettings.unpinnedWindow && !this.guiSettings.startMinimized;
      default:
        return true;
    }
  }

  private getProblemReportPath(id: string): string {
    return path.join(app.getPath('temp'), `${id}.log`);
  }

  private async updateMacOsScrollbarVisibility(): Promise<void> {
    const command =
      'defaults read kCFPreferencesAnyApplication AppleShowScrollBars || echo Automatic';
    const { stdout } = await execAsync(command);
    switch (stdout.trim()) {
      case 'WhenScrolling':
        this.macOsScrollbarVisibility = MacOsScrollbarVisibility.whenScrolling;
        break;
      case 'Always':
        this.macOsScrollbarVisibility = MacOsScrollbarVisibility.always;
        break;
      case 'Automatic':
      default:
        this.macOsScrollbarVisibility = MacOsScrollbarVisibility.automatic;
        break;
    }

    IpcMainEventChannel.window.notifyMacOsScrollbarVisibility?.(this.macOsScrollbarVisibility);
  }

  /* eslint-disable @typescript-eslint/member-ordering */
  // NotificationControllerDelagate
  public openApp = () => this.userInterface?.showWindow();
  public openLink = async (url: string, withAuth?: boolean) => {
    if (withAuth) {
      let token = '';
      try {
        token = await this.daemonRpc.getWwwAuthToken();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to get the WWW auth token: ${error.message}`);
      }
      return shell.openExternal(`${url}?token=${token}`);
    } else {
      return shell.openExternal(url);
    }
  };
  public isWindowVisible = () => this.userInterface?.isWindowVisible() ?? false;
  public areSystemNotificationsEnabled = () => this.guiSettings.enableSystemNotifications;

  // UserInterfaceDelegate
  public cancelPendingNotifications = () =>
    this.notificationController.cancelPendingNotifications();
  public resetTunnelStateAnnouncements = () =>
    this.notificationController.resetTunnelStateAnnouncements();
  public getUnpinnedWindow = () => this.guiSettings.unpinnedWindow;
  public checkVolumes = () => this.daemonRpc.checkVolumes();
  public getAppQuitStage = () => this.quitStage;
  public isConnectedToDaemon = () => this.connectedToDaemon;
  public getTunnelState = () => this.tunnelState;
  public updateAccountData = () => {
    if (this.connectedToDaemon && this.isLoggedIn()) {
      this.accountDataCache.fetch(this.getAccountToken()!);
    }
  };
  public isLoggedIn(): boolean {
    return this.deviceState?.type === 'logged in';
  }
  public isBrowsingFiles = () => this.browsingFiles;
  public getAccountData = () => this.accountData;

  public connectTunnel = async (): Promise<void> => {
    if (connectEnabled(this.connectedToDaemon, this.isLoggedIn(), this.tunnelState.state)) {
      this.setOptimisticTunnelState('connecting');
      await this.daemonRpc.connectTunnel();
    }
  };

  public reconnectTunnel = async (): Promise<void> => {
    if (reconnectEnabled(this.connectedToDaemon, this.isLoggedIn(), this.tunnelState.state)) {
      this.setOptimisticTunnelState('connecting');
      await this.daemonRpc.reconnectTunnel();
    }
  };

  public disconnectTunnel = async (): Promise<void> => {
    if (disconnectEnabled(this.connectedToDaemon, this.tunnelState.state)) {
      this.setOptimisticTunnelState('disconnecting');
      await this.daemonRpc.disconnectTunnel();
    }
  };

  // VersionDelegate
  public notify = (notification: SystemNotification) => {
    this.notificationController.notify(notification);
  };
  public getVersionInfo = () => this.daemonRpc.getVersionInfo();

  /* eslint-enable @typescript-eslint/member-ordering */
}

const applicationMain = new ApplicationMain();
applicationMain.run();
