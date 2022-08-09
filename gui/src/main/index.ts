import { exec } from 'child_process';
import { app, dialog, nativeTheme, session, shell, systemPreferences } from 'electron';
import fs from 'fs';
import * as path from 'path';
import util from 'util';

import config from '../config.json';
import { hasExpired } from '../shared/account-expiry';
import { IWindowsApplication } from '../shared/application-types';
import { DaemonEvent, DeviceEvent, ISettings, TunnelState } from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import { SYSTEM_PREFERRED_LOCALE_KEY } from '../shared/gui-settings-state';
import { ITranslations, MacOsScrollbarVisibility } from '../shared/ipc-schema';
import { IChangelog, IHistoryObject, ScrollPositions } from '../shared/ipc-types';
import log, { ConsoleOutput, Logger } from '../shared/logging';
import { LogLevel } from '../shared/logging-types';
import { SystemNotification } from '../shared/notifications/notification';
import Account, { AccountDelegate } from './account';
import { getOpenAtLogin } from './autostart';
import { readChangelog } from './changelog';
import { ConnectionObserver, DaemonRpc, SubscriptionListener } from './daemon-rpc';
import Expectation from './expectation';
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
import * as problemReport from './problem-report';
import ReconnectionBackoff from './reconnection-backoff';
import RelayList from './relay-list';
import Settings, { SettingsDelegate } from './settings';
import TunnelStateHandler, { TunnelStateHandlerDelegate } from './tunnel-state';
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
  implements
    NotificationControllerDelegate,
    UserInterfaceDelegate,
    VersionDelegate,
    TunnelStateHandlerDelegate,
    SettingsDelegate,
    AccountDelegate {
  private daemonRpc = new DaemonRpc();
  private notificationController = new NotificationController(this);
  private version = new Version(this, UPDATE_NOTIFICATION_DISABLED);
  private settings = new Settings(this, this.daemonRpc, this.version.currentVersion);
  private relayList = new RelayList();
  private userInterface?: UserInterface;
  private account: Account = new Account(this, this.daemonRpc);
  private tunnelState = new TunnelStateHandler(this);

  // True while file pickers are displayed which is used to decide if the Browser window should be
  // hidden when losing focus.
  private browsingFiles = false;

  private daemonEventListener?: SubscriptionListener<DaemonEvent>;
  private reconnectBackoff = new ReconnectionBackoff();
  private beforeFirstDaemonConnection = true;
  private isPerformingPostUpgrade = false;
  private quitStage = AppQuitStage.unready;

  private tunnelStateExpectation?: Expectation;

  // The UI locale which is set once from onReady handler
  private locale = 'en';

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

    this.settings.gui.load();
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

  public async performPostUpgradeCheck(): Promise<void> {
    const oldValue = this.isPerformingPostUpgrade;
    this.isPerformingPostUpgrade = await this.daemonRpc.isPerformingPostUpgrade();
    if (this.isPerformingPostUpgrade !== oldValue) {
      IpcMainEventChannel.daemon.notifyIsPerformingPostUpgrade?.(this.isPerformingPostUpgrade);
    }
  }

  public connectTunnel = async (): Promise<void> => {
    if (this.tunnelState.allowConnect(this.daemonRpc.isConnected, this.account.isLoggedIn())) {
      this.tunnelState.expectNextTunnelState('connecting');
      await this.daemonRpc.connectTunnel();
    }
  };

  public reconnectTunnel = async (): Promise<void> => {
    if (this.tunnelState.allowReconnect(this.daemonRpc.isConnected, this.account.isLoggedIn())) {
      this.tunnelState.expectNextTunnelState('connecting');
      await this.daemonRpc.reconnectTunnel();
    }
  };

  public disconnectTunnel = async (): Promise<void> => {
    if (this.tunnelState.allowDisconnect(this.daemonRpc.isConnected)) {
      this.tunnelState.expectNextTunnelState('disconnecting');
      await this.daemonRpc.disconnectTunnel();
    }
  };

  public isLoggedIn = () => this.account.isLoggedIn();

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
      if (this.daemonRpc.isConnected) {
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

    if (this.daemonRpc.isConnected) {
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
    const preferredLocale = this.settings.gui.preferredLocale;
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

    this.userInterface = new UserInterface(
      this,
      this.daemonRpc,
      SANDBOX_DISABLED,
      NAVIGATION_RESET_DISABLED,
    );

    this.tunnelStateExpectation = new Expectation(async () => {
      this.userInterface?.createTrayIconController(
        this.tunnelState.tunnelState,
        this.settings.blockWhenDisconnected,
        this.settings.gui.monochromaticIcon,
      );
      await this.userInterface?.updateTrayTheme();

      this.userInterface?.setTrayContextMenu();
      this.userInterface?.setTrayTooltip();

      if (process.platform === 'win32') {
        nativeTheme.on('updated', async () => {
          if (this.settings.gui.monochromaticIcon) {
            await this.userInterface?.updateTrayTheme();
          }
        });
      }
    });

    this.registerIpcListeners();

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
      this.account.setAccountHistory(await this.daemonRpc.getAccountHistory());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the account history: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch the tunnel state
    try {
      this.tunnelState.handleNewTunnelState(await this.daemonRpc.getState());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the tunnel state: ${error.message}`);

      return this.handleBootstrapError(error);
    }

    // fetch device
    try {
      const deviceState = await this.daemonRpc.getDevice();
      this.account.handleDeviceEvent({ type: deviceState.type, deviceState } as DeviceEvent);
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
      this.relayList.setRelays(
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

    // notify renderer, this.daemonRpc.isConnected could have changed if the daemon disconnected
    // again before this if-statement is reached.
    if (this.daemonRpc.isConnected) {
      IpcMainEventChannel.daemon.notifyConnected?.();
    }

    if (firstDaemonConnection) {
      void this.autoConnect();
    }

    // show window when account is not set
    if (!this.account.isLoggedIn()) {
      this.userInterface?.showWindow();
    }
  };

  private onDaemonDisconnected = (error?: Error) => {
    if (this.daemonEventListener) {
      this.daemonRpc.unsubscribeDaemonEventListener(this.daemonEventListener);
    }
    // make sure we were connected before to distinguish between a failed attempt to reconnect and
    // connection loss.
    const wasConnected = this.daemonRpc.isConnected;

    // Reset the daemon event listener since it's going to be invalidated on disconnect
    this.daemonEventListener = undefined;

    this.tunnelState.resetFallback();

    if (wasConnected) {
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
          this.tunnelState.handleNewTunnelState(daemonEvent.tunnelState);
        } else if ('settings' in daemonEvent) {
          this.setSettings(daemonEvent.settings);
        } else if ('relayList' in daemonEvent) {
          this.relayList.setRelays(
            daemonEvent.relayList,
            this.settings.relaySettings,
            this.settings.bridgeState,
          );
        } else if ('appVersionInfo' in daemonEvent) {
          this.version.setLatestVersion(daemonEvent.appVersionInfo);
        } else if ('device' in daemonEvent) {
          this.account.handleDeviceEvent(daemonEvent.device);
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

  private setSettings(newSettings: ISettings) {
    const oldSettings = this.settings;
    this.settings.handleNewSettings(newSettings);

    this.userInterface?.updateTrayIcon(
      this.tunnelState.tunnelState,
      newSettings.blockWhenDisconnected,
    );

    if (oldSettings.showBetaReleases !== newSettings.showBetaReleases) {
      this.version.setLatestVersion(this.version.upgradeVersion);
    }

    IpcMainEventChannel.settings.notify?.(newSettings);

    if (windowsSplitTunneling) {
      void this.updateSplitTunnelingApplications(newSettings.splitTunnel.appsList);
    }

    // since settings can have the relay constraints changed, the relay
    // list should also be updated
    this.relayList.updateSettings(newSettings.relaySettings, newSettings.bridgeState);
  }

  private async updateSplitTunnelingApplications(appList: string[]): Promise<void> {
    const { applications } = await windowsSplitTunneling.getApplications({
      applicationPaths: appList,
    });
    this.windowsSplitTunnelingApplications = applications;

    IpcMainEventChannel.windowsSplitTunneling.notify?.(applications);
  }

  private registerIpcListeners() {
    IpcMainEventChannel.state.handleGet(() => ({
      isConnected: this.daemonRpc.isConnected,
      autoStart: getOpenAtLogin(),
      accountData: this.account.accountData,
      accountHistory: this.account.accountHistory,
      tunnelState: this.tunnelState.tunnelState,
      settings: this.settings.all,
      isPerformingPostUpgrade: this.isPerformingPostUpgrade,
      deviceState: this.account.deviceState,
      relayListPair: this.relayList.getProcessedRelays(
        this.settings.relaySettings,
        this.settings.bridgeState,
      ),
      currentVersion: this.version.currentVersion,
      upgradeVersion: this.version.upgradeVersion,
      guiSettings: this.settings.gui.state,
      translations: this.translations,
      windowsSplitTunnelingApplications: this.windowsSplitTunnelingApplications,
      macOsScrollbarVisibility: this.macOsScrollbarVisibility,
      changelog: this.changelog ?? [],
      forceShowChanges: FORCE_SHOW_CHANGES,
      navigationHistory: this.navigationHistory,
      scrollPositions: this.scrollPositions,
    }));

    IpcMainEventChannel.location.handleGet(() => this.daemonRpc.getLocation());

    IpcMainEventChannel.tunnel.handleConnect(this.connectTunnel);
    IpcMainEventChannel.tunnel.handleReconnect(this.reconnectTunnel);
    IpcMainEventChannel.tunnel.handleDisconnect(this.disconnectTunnel);

    IpcMainEventChannel.guiSettings.handleSetPreferredLocale((locale: string) => {
      this.settings.gui.preferredLocale = locale;
      this.updateCurrentLocale();
      return Promise.resolve(this.translations);
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
        this.settings.gui.addBrowsedForSplitTunnelingApplications(application);
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
        this.settings.gui.deleteBrowsedForSplitTunnelingApplications(application.absolutepath);
        return windowsSplitTunneling.removeApplicationFromCache(application);
      },
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

    IpcMainEventChannel.navigation.handleSetHistory((history) => {
      this.navigationHistory = history;
    });
    IpcMainEventChannel.navigation.handleSetScrollPositions((scrollPositions) => {
      this.scrollPositions = scrollPositions;
    });

    problemReport.registerIpcListeners();
    this.settings.registerIpcListeners();
    this.account.registerIpcListeners();

    if (windowsSplitTunneling) {
      this.settings.gui.browsedForSplitTunnelingApplications.forEach(
        windowsSplitTunneling.addApplicationPathToCache,
      );
    }
  }

  private async autoConnect() {
    if (process.env.NODE_ENV === 'development') {
      log.info('Skip autoconnect in development');
    } else if (
      this.account.isLoggedIn() &&
      (!this.account.accountData || !hasExpired(this.account.accountData.expiry))
    ) {
      if (this.settings.gui.autoConnect) {
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
    return this.settings.gui.unpinnedWindow && !this.settings.gui.startMinimized;
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
  public areSystemNotificationsEnabled = () => this.settings.gui.enableSystemNotifications;

  // UserInterfaceDelegate
  public cancelPendingNotifications = () =>
    this.notificationController.cancelPendingNotifications();
  public resetTunnelStateAnnouncements = () =>
    this.notificationController.resetTunnelStateAnnouncements();
  public isUnpinnedWindow = () => this.settings.gui.unpinnedWindow;
  public checkVolumes = () => this.daemonRpc.checkVolumes();
  public getAppQuitStage = () => this.quitStage;
  public isConnectedToDaemon = () => this.daemonRpc.isConnected;
  public getTunnelState = () => this.tunnelState.tunnelState;
  public updateAccountData = () => this.account.updateAccountData();
  public isBrowsingFiles = () => this.browsingFiles;
  public getAccountData = () => this.account.accountData;

  // VersionDelegate
  public notify = (notification: SystemNotification) => {
    this.notificationController.notify(notification);
  };
  public getVersionInfo = () => this.daemonRpc.getVersionInfo();

  // TunnelStateHandlerDelegate
  public handleTunnelStateUpdate = (tunnelState: TunnelState) => {
    this.userInterface?.updateTrayIcon(tunnelState, this.settings.blockWhenDisconnected);

    this.userInterface?.setTrayContextMenu();
    this.userInterface?.setTrayTooltip();

    this.notificationController.notifyTunnelState(
      tunnelState,
      this.settings.blockWhenDisconnected,
      this.settings.splitTunnel.enableExclusions && this.settings.splitTunnel.appsList.length > 0,
      this.account.accountData?.expiry,
    );

    IpcMainEventChannel.tunnel.notify?.(tunnelState);

    if (this.account.accountData) {
      this.account.detectStaleAccountExpiry(tunnelState);
    }
  };

  // SettingsDelegate
  public handleMonochromaticIconChange = (value: boolean) =>
    this.userInterface?.setUseMonochromaticTrayIcon(value) ?? Promise.resolve();
  public handleUnpinnedWindowChange = () => void this.userInterface?.recreateWindow();

  // AccountDelegate
  public getLocale = () => this.locale;
  public isPerformingPostUpgradeCheck = () => this.isPerformingPostUpgrade;
  public setTrayContextMenu = () => this.userInterface?.setTrayContextMenu();
  /* eslint-enable @typescript-eslint/member-ordering */
}

const applicationMain = new ApplicationMain();
applicationMain.run();
