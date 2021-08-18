import { execFile } from 'child_process';
import {
  app,
  BrowserWindow,
  dialog,
  Menu,
  nativeImage,
  screen,
  session,
  shell,
  Tray,
} from 'electron';
import os from 'os';
import * as path from 'path';
import { sprintf } from 'sprintf-js';
import * as uuid from 'uuid';
import config from '../config.json';
import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import { IApplication } from '../shared/application-types';
import BridgeSettingsBuilder from '../shared/bridge-settings-builder';
import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  DaemonEvent,
  IAccountData,
  IAppVersionInfo,
  IDnsOptions,
  ILocation,
  IRelayList,
  ISettings,
  IWireguardPublicKey,
  KeygenEvent,
  liftConstraint,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import { SYSTEM_PREFERRED_LOCALE_KEY } from '../shared/gui-settings-state';
import log, { ConsoleOutput, Logger } from '../shared/logging';
import { LogLevel } from '../shared/logging-types';
import { IpcMainEventChannel } from './ipc-event-channel';
import { ICurrentAppVersionInfo } from '../shared/ipc-types';
import {
  AccountExpiredNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  InconsistentVersionNotificationProvider,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../shared/notifications/notification';
import { Scheduler } from '../shared/scheduler';
import AccountDataCache from './account-data-cache';
import { getOpenAtLogin, setOpenAtLogin } from './autostart';
import { ConnectionObserver, DaemonRpc, SubscriptionListener } from './daemon-rpc';
import { InvalidAccountError } from './errors';
import Expectation from './expectation';
import GuiSettings from './gui-settings';
import { findIconPath } from './linux-desktop-entry';
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
import { loadTranslations } from './load-translations';
import NotificationController from './notification-controller';
import { resolveBin } from './proc';
import ReconnectionBackoff from './reconnection-backoff';
import TrayIconController, { TrayIconType } from './tray-icon-controller';
import WindowController from './window-controller';
import { ITranslations } from '../shared/ipc-schema';

// Only import split tunneling library on correct OS.
const linuxSplitTunneling = process.platform === 'linux' && require('./linux-split-tunneling');
const windowsSplitTunneling = process.platform === 'win32' && require('./windows-split-tunneling');

const DAEMON_RPC_PATH =
  process.platform === 'win32' ? 'unix:////./pipe/Mullvad VPN' : 'unix:///var/run/mullvad-vpn';

const AUTO_CONNECT_FALLBACK_DELAY = 6000;

const GUI_VERSION = app.getVersion().replace('.0', '');
/// Mirrors the beta check regex in the daemon. Matches only well formed beta versions
const IS_BETA = /^(\d{4})\.(\d+)-beta(\d+)$/;

const SANDBOX_DISABLED = app.commandLine.hasSwitch('no-sandbox');

enum AppQuitStage {
  unready,
  initiated,
  ready,
}

type AccountVerification = { status: 'verified' } | { status: 'deferred'; error: Error };

class ApplicationMain {
  private notificationController = new NotificationController({
    openApp: () => this.windowController?.show(),
    openLink: (url: string, withAuth?: boolean) => this.openLink(url, withAuth),
    isWindowVisible: () => this.windowController?.isVisible() ?? false,
    areSystemNotificationsEnabled: () => this.guiSettings.enableSystemNotifications,
  });
  private windowController?: WindowController;
  private tray?: Tray;
  private trayIconController?: TrayIconController;

  // True while file pickers are displayed which is used to decide if the Browser window should be
  // hidden when losing focus.
  private browsingFiles = false;

  private daemonRpc = new DaemonRpc(DAEMON_RPC_PATH);
  private daemonEventListener?: SubscriptionListener<DaemonEvent>;
  private reconnectBackoff = new ReconnectionBackoff();
  private connectedToDaemon = false;
  private quitStage = AppQuitStage.unready;

  private accountData?: IAccountData = undefined;
  private accountHistory?: AccountToken = undefined;
  private tunnelState: TunnelState = { state: 'disconnected' };
  private settings: ISettings = {
    accountToken: undefined,
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
        openvpnConstraints: {
          port: 'any',
          protocol: 'any',
        },
        wireguardConstraints: {
          port: 'any',
        },
      },
    },
    bridgeSettings: {
      normal: {
        location: 'any',
        providers: [],
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
        },
        customOptions: {
          addresses: [],
        },
      },
    },
  };
  private guiSettings = new GuiSettings();
  private location?: ILocation;
  private lastDisconnectedLocation?: ILocation;
  private tunnelStateExpectation?: Expectation;
  private getLocationPromise?: Promise<ILocation>;

  private relays: IRelayList = { countries: [] };

  private currentVersion: ICurrentAppVersionInfo = {
    daemon: undefined,
    gui: GUI_VERSION,
    isConsistent: true,
    isBeta: IS_BETA.test(GUI_VERSION),
  };

  private upgradeVersion: IAppVersionInfo = {
    supported: true,
    suggestedUpgrade: undefined,
  };

  // The UI locale which is set once from onReady handler
  private locale = 'en';

  private wireguardPublicKey?: IWireguardPublicKey;

  private accountExpiryNotificationScheduler = new Scheduler();

  private accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this.daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      this.accountData = accountData && {
        ...accountData,
        previousExpiry:
          accountData.expiry !== this.accountData?.expiry
            ? this.accountData?.expiry
            : this.accountData?.previousExpiry,
      };

      if (this.windowController) {
        IpcMainEventChannel.account.notify(this.windowController.webContents, this.accountData);
      }

      this.handleAccountExpiry();
    },
  );

  private autoConnectOnWireguardKeyEvent = false;
  private autoConnectFallbackScheduler = new Scheduler();

  private rendererLog?: Logger;
  private translations: ITranslations = { locale: this.locale };

  private windowsSplitTunnelingApplications?: IApplication[];

  public run() {
    // Remove window animations to combat window flickering when opening window. Can be removed when
    // this issue has been resolved: https://github.com/electron/electron/issues/12130
    if (process.platform === 'win32') {
      app.commandLine.appendSwitch('wm-window-animations-disabled');
    }

    this.overrideAppPaths();

    if (this.ensureSingleInstance()) {
      return;
    }

    this.initLogging();

    log.debug(`Chromium sandbox is ${SANDBOX_DISABLED ? 'disabled' : 'enabled'}`);
    if (!SANDBOX_DISABLED) {
      app.enableSandbox();
    }

    log.info(`Running version ${this.currentVersion.gui}`);

    if (process.platform === 'win32') {
      app.setAppUserModelId('net.mullvad.vpn');
    }

    // While running in development the watch script triggers a reload of the renderer by sending
    // the signal `SIGUSR2`.
    if (process.env.NODE_ENV === 'development') {
      process.on('SIGUSR2', () => {
        this.windowController?.window?.reload();
      });
    }

    this.guiSettings.load();

    app.on('activate', this.onActivate);
    app.on('ready', this.onReady);
    app.on('window-all-closed', () => app.quit());
    app.on('before-quit', this.onBeforeQuit);
  }

  private ensureSingleInstance() {
    if (app.requestSingleInstanceLock()) {
      app.on('second-instance', (_event, _commandLine, _workingDirectory) => {
        if (this.windowController) {
          this.windowController.show();
        }
      });
      return false;
    } else {
      app.quit();
      return true;
    }
  }

  private overrideAppPaths() {
    // This ensures that on Windows the %LOCALAPPDATA% directory is used instead of the %ADDDATA%
    // directory that has roaming contents
    if (process.platform === 'win32') {
      const appDataDir = process.env.LOCALAPPDATA;
      if (appDataDir) {
        app.setPath('appData', appDataDir);
        app.setPath('userData', path.join(appDataDir, app.name));
        app.setPath('logs', path.join(appDataDir, app.name, 'logs'));
      } else {
        throw new Error('Missing %LOCALAPPDATA% environment variable');
      }
    } else if (process.platform === 'linux') {
      const userDataDir = app.getPath('userData');
      app.setPath('logs', path.join(userDataDir, 'logs'));
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

        log.addOutput(new FileOutput(LogLevel.debug, mainLogPath));
        this.rendererLog.addOutput(new FileOutput(LogLevel.debug, rendererLogPath));
      } catch (e) {
        console.error('Failed to initialize logging:', e);
      }
    }

    log.addOutput(new ConsoleOutput(LogLevel.debug));
  }

  private onActivate = () => {
    if (this.windowController) {
      this.windowController.show();
    }
  };

  private onBeforeQuit = async (event: Electron.Event) => {
    switch (this.quitStage) {
      case AppQuitStage.unready:
        // postpone the app shutdown
        event.preventDefault();

        this.quitStage = AppQuitStage.initiated;
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
    if (this.connectedToDaemon) {
      try {
        await this.daemonRpc.disconnectTunnel();
        log.info('Disconnected the tunnel');
      } catch (e) {
        log.error(`Failed to disconnect the tunnel: ${e.message}`);
      }
    } else {
      log.info('Cannot close the tunnel because there is no active connection to daemon.');
    }

    // Unsubscribe the event handler
    try {
      if (this.daemonEventListener) {
        this.daemonRpc.unsubscribeDaemonEventListener(this.daemonEventListener);

        log.info('Unsubscribed from the daemon events');
      }
    } catch (e) {
      log.error(`Failed to unsubscribe from daemon events: ${e.message}`);
    }

    // The window is not closable on macOS to be able to hide the titlebar and workaround
    // a shadow bug rendered above the invisible title bar. This also prevents the window from
    // closing normally, even programmatically. Therefore re-enable the close button just before
    // quitting the app.
    // Github issue: https://github.com/electron/electron/issues/15008
    if (process.platform === 'darwin' && this.windowController?.window) {
      this.windowController.window.closable = true;
    }

    this.daemonRpc.disconnect();

    for (const logger of [log, this.rendererLog]) {
      try {
        logger?.dispose();
      } catch (e) {
        console.error('Failed to dispose logger:', e);
      }
    }
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
    // There's no option that prevents Electron from fetching spellcheck dictionaries from
    // Chromium's CDN and passing a non-resolving URL is the only known way to prevent it from
    // fetching.  https://github.com/electron/electron/issues/22995
    session.defaultSession.setSpellCheckerDictionaryDownloadURL('https://00.00/');

    // Blocks scripts in the renderer process from asking for any permission.
    this.blockPermissionRequests();
    // Blocks any http(s) and file requests that aren't supposed to happen.
    this.blockRequests();
    // Blocks navigation since it's not needed.
    this.blockNavigation();

    this.updateCurrentLocale();

    this.daemonRpc.addConnectionObserver(
      new ConnectionObserver(this.onDaemonConnected, this.onDaemonDisconnected),
    );
    this.connectToDaemon();

    const window = await this.createWindow();
    const tray = this.createTray();

    const windowController = new WindowController(window, tray, this.guiSettings.unpinnedWindow);
    this.tunnelStateExpectation = new Expectation(() => {
      this.trayIconController = new TrayIconController(
        tray,
        this.trayIconType(this.tunnelState, this.settings.blockWhenDisconnected),
        this.guiSettings.monochromaticIcon,
      );
    });

    this.registerIpcListeners();

    this.windowController = windowController;
    this.tray = tray;

    this.guiSettings.onChange = (newState, oldState) => {
      if (oldState.monochromaticIcon !== newState.monochromaticIcon) {
        if (this.trayIconController) {
          this.trayIconController.useMonochromaticIcon = newState.monochromaticIcon;
        }
      }

      if (newState.autoConnect !== oldState.autoConnect) {
        this.updateDaemonsAutoConnect();
      }

      if (this.windowController) {
        IpcMainEventChannel.guiSettings.notify(this.windowController.webContents, newState);
      }
    };

    if (this.shouldShowWindowOnStart() || process.env.NODE_ENV === 'development') {
      windowController.show();
    }

    await this.initializeWindow();
  };

  private async initializeWindow() {
    if (this.windowController?.window && this.tray) {
      this.registerWindowListener(this.windowController);
      this.addContextMenu(this.windowController);

      if (process.env.NODE_ENV === 'development') {
        await this.installDevTools();
        this.windowController.window.webContents.openDevTools({ mode: 'detach' });
      }

      switch (process.platform) {
        case 'win32':
          this.installWindowsMenubarAppWindowHandlers(this.tray, this.windowController);
          break;
        case 'darwin':
          this.installMacOsMenubarAppWindowHandlers(this.windowController);
          this.setMacOsAppMenu();
          break;
        case 'linux':
          this.tray.setContextMenu(this.createTrayContextMenu());
          this.setLinuxAppMenu();
          this.windowController.window.setMenuBarVisibility(false);
          break;
      }

      this.installWindowCloseHandler(this.windowController);
      this.installTrayClickHandlers();
      this.installGenericFocusHandlers(this.windowController);

      const filePath = path.resolve(path.join(__dirname, '../renderer/index.html'));
      try {
        await this.windowController.window.loadFile(filePath);
      } catch (error) {
        log.error(`Failed to load index file: ${error.message}`);
      }

      // disable pinch to zoom
      if (this.windowController.webContents) {
        void this.windowController.webContents.setVisualZoomLevelLimits(1, 1);
      }
    }
  }

  private onDaemonConnected = async () => {
    this.connectedToDaemon = true;

    log.info('Connected to the daemon');

    // subscribe to events
    try {
      this.daemonEventListener = this.subscribeEvents();
    } catch (error) {
      log.error(`Failed to subscribe: ${error.message}`);

      return this.recoverFromBootstrapError(error);
    }

    // fetch account history
    try {
      this.setAccountHistory(await this.daemonRpc.getAccountHistory());
    } catch (error) {
      log.error(`Failed to fetch the account history: ${error.message}`);

      return this.recoverFromBootstrapError(error);
    }

    // fetch the tunnel state
    try {
      this.setTunnelState(await this.daemonRpc.getState());
    } catch (error) {
      log.error(`Failed to fetch the tunnel state: ${error.message}`);

      return this.recoverFromBootstrapError(error);
    }

    // fetch settings
    try {
      this.setSettings(await this.daemonRpc.getSettings());
    } catch (error) {
      log.error(`Failed to fetch settings: ${error.message}`);

      return this.recoverFromBootstrapError(error);
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
    } catch (error) {
      log.error(`Failed to fetch relay locations: ${error.message}`);

      return this.recoverFromBootstrapError(error);
    }

    // fetch the daemon's version
    try {
      this.setDaemonVersion(await this.daemonRpc.getCurrentVersion());
    } catch (error) {
      log.error(`Failed to fetch the daemon's version: ${error.message}`);

      return this.recoverFromBootstrapError(error);
    }

    // fetch the latest version info in background
    void this.fetchLatestVersion();

    // reset the reconnect backoff when connection established.
    this.reconnectBackoff.reset();

    // notify renderer, this.connectedToDaemon could have changed if the daemon disconnected again
    // before this if-statement is reached.
    if (this.windowController && this.connectedToDaemon) {
      IpcMainEventChannel.daemon.notifyConnected(this.windowController.webContents);
    }

    // show window when account is not set
    if (!this.settings.accountToken) {
      this.windowController?.show();
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

    if (wasConnected) {
      this.connectedToDaemon = false;

      // update the tray icon to indicate that the computer is not secure anymore
      this.updateTrayIcon({ state: 'disconnected' }, false);

      // notify renderer process
      if (this.windowController) {
        IpcMainEventChannel.daemon.notifyDisconnected(this.windowController.webContents);
      }
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
    void this.daemonRpc.connect();
  }

  private recoverFromBootstrapError(_error?: Error) {
    // Attempt to reconnect to daemon if the program fails to fetch settings, tunnel state or
    // subscribe for RPC events.
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
        } else if ('wireguardKey' in daemonEvent) {
          this.handleWireguardKeygenEvent(daemonEvent.wireguardKey);
        } else if ('appVersionInfo' in daemonEvent) {
          this.setLatestVersion(daemonEvent.appVersionInfo);
        }
      },
      (error: Error) => {
        log.error(`Cannot deserialize the daemon event: ${error.message}`);
        log.error(error.stack);
      },
    );

    this.daemonRpc.subscribeDaemonEventListener(daemonEventListener);

    return daemonEventListener;
  }

  private setAccountHistory(accountHistory?: AccountToken) {
    this.accountHistory = accountHistory;

    if (this.windowController) {
      IpcMainEventChannel.accountHistory.notify(this.windowController.webContents, accountHistory);
    }
  }

  private setWireguardKey(wireguardKey?: IWireguardPublicKey) {
    this.wireguardPublicKey = wireguardKey;
    if (this.windowController) {
      IpcMainEventChannel.wireguardKeys.notifyPublicKey(
        this.windowController.webContents,
        wireguardKey,
      );
    }

    if (wireguardKey) {
      this.wireguardKeygenEventAutoConnect();
    }
  }

  private handleWireguardKeygenEvent(event: KeygenEvent) {
    switch (event) {
      case 'too_many_keys':
      case 'generation_failure':
        this.wireguardPublicKey = undefined;
        break;
      default:
        this.wireguardPublicKey = event.newKey;
    }

    if (this.windowController) {
      IpcMainEventChannel.wireguardKeys.notifyKeygenEvent(this.windowController.webContents, event);
    }

    this.wireguardKeygenEventAutoConnect();
  }

  private setTunnelState(newState: TunnelState) {
    this.tunnelState = newState;
    this.updateTrayIcon(newState, this.settings.blockWhenDisconnected);
    void this.updateLocation();

    if (process.platform === 'linux') {
      this.tray?.setContextMenu(this.createTrayContextMenu());
    }

    this.notificationController.notifyTunnelState(
      newState,
      this.settings.blockWhenDisconnected,
      this.settings.splitTunnel.enableExclusions && this.settings.splitTunnel.appsList.length > 0,
      this.accountData?.expiry,
    );

    if (this.windowController) {
      IpcMainEventChannel.tunnel.notify(this.windowController.webContents, newState);
    }

    if (this.accountData) {
      this.detectStaleAccountExpiry(newState, new Date(this.accountData.expiry));
    }
  }

  private setSettings(newSettings: ISettings) {
    const oldSettings = this.settings;
    this.settings = newSettings;

    this.updateTrayIcon(this.tunnelState, newSettings.blockWhenDisconnected);

    // make sure to invalidate the account data cache when account tokens change
    this.updateAccountDataOnAccountChange(oldSettings.accountToken, newSettings.accountToken);

    if (oldSettings.accountToken !== newSettings.accountToken) {
      void this.updateAccountHistory();
      void this.fetchWireguardKey();
    }

    if (oldSettings.showBetaReleases !== newSettings.showBetaReleases) {
      this.setLatestVersion(this.upgradeVersion);
    }

    if (this.windowController) {
      IpcMainEventChannel.settings.notify(this.windowController.webContents, newSettings);

      if (windowsSplitTunneling) {
        void this.updateSplitTunnelingApplications(newSettings.splitTunnel.appsList);
      }
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

    if (this.windowController) {
      IpcMainEventChannel.windowsSplitTunneling.notify(
        this.windowController.webContents,
        applications,
      );
    }
  }

  private setLocation(newLocation: ILocation) {
    this.location = newLocation;

    if (this.windowController) {
      IpcMainEventChannel.location.notify(this.windowController.webContents, newLocation);
    }
  }

  private setRelays(
    newRelayList: IRelayList,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
  ) {
    this.relays = newRelayList;

    const filteredRelays = this.processRelaysForPresentation(
      newRelayList,
      relaySettings,
      bridgeState,
    );
    const filteredBridges = this.processBridgesForPresentation(newRelayList, bridgeState);

    if (this.windowController) {
      IpcMainEventChannel.relays.notify(this.windowController.webContents, {
        relays: filteredRelays,
        bridges: filteredBridges,
      });
    }
  }

  private processRelaysForPresentation(
    relayList: IRelayList,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
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
              if (relay.tunnels) {
                switch (tunnelProtocol) {
                  case 'openvpn':
                    return relay.tunnels.openvpn.length > 0;

                  case 'wireguard':
                    return relay.tunnels.wireguard.length > 0;

                  case 'any':
                    // TODO: Drop win32 check when Wireguard becomes default on Windows
                    if (process.platform === 'win32' || bridgeState === 'on') {
                      return relay.tunnels.openvpn.length > 0;
                    } else {
                      return relay.tunnels.openvpn.length > 0 || relay.tunnels.wireguard.length > 0;
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
              relays: city.relays.filter(
                (relay) => relay.bridges && relay.bridges.shadowsocks.length > 0,
              ),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0);

      return { countries: filteredCountries };
    } else {
      return { countries: [] };
    }
  }

  private setDaemonVersion(daemonVersion: string) {
    const versionInfo = {
      ...this.currentVersion,
      daemon: daemonVersion,
      isConsistent: daemonVersion === this.currentVersion.gui,
    };

    this.currentVersion = versionInfo;

    if (!versionInfo.isConsistent) {
      log.info('Inconsistent version', {
        guiVersion: versionInfo.gui,
        daemonVersion: versionInfo.daemon,
      });
    }

    // notify user about inconsistent version
    const notificationProvider = new InconsistentVersionNotificationProvider({
      consistent: versionInfo.isConsistent,
    });
    if (notificationProvider.mayDisplay()) {
      this.notificationController.notify(notificationProvider.getSystemNotification());
    }

    // notify renderer
    if (this.windowController) {
      IpcMainEventChannel.currentVersion.notify(this.windowController.webContents, versionInfo);
    }
  }

  private setLatestVersion(latestVersionInfo: IAppVersionInfo) {
    const suggestedIsBeta =
      latestVersionInfo.suggestedUpgrade !== undefined &&
      IS_BETA.test(latestVersionInfo.suggestedUpgrade);

    const upgradeVersion = {
      ...latestVersionInfo,
      suggestedIsBeta,
    };

    this.upgradeVersion = upgradeVersion;

    // notify user to update the app if it became unsupported
    const notificationProviders = [
      new UnsupportedVersionNotificationProvider({
        supported: latestVersionInfo.supported,
        consistent: this.currentVersion.isConsistent,
        suggestedUpgrade: latestVersionInfo.suggestedUpgrade,
        suggestedIsBeta,
      }),
      new UpdateAvailableNotificationProvider({
        suggestedUpgrade: latestVersionInfo.suggestedUpgrade,
        suggestedIsBeta,
      }),
    ];
    const notificationProvider = notificationProviders.find((notificationProvider) =>
      notificationProvider.mayDisplay(),
    );
    if (notificationProvider) {
      this.notificationController.notify(notificationProvider.getSystemNotification());
    }

    if (this.windowController) {
      IpcMainEventChannel.upgradeVersion.notify(this.windowController.webContents, upgradeVersion);
    }
  }

  private async fetchLatestVersion() {
    try {
      this.setLatestVersion(await this.daemonRpc.getVersionInfo());
    } catch (error) {
      log.error(`Failed to request the version info: ${error.message}`);
    }
  }

  private async updateLocation() {
    const tunnelState = this.tunnelState;

    if (tunnelState.state === 'connected' || tunnelState.state === 'connecting') {
      // Location was broadcasted with the tunnel state, but it doesn't contain the relay out IP
      // address, so it will have to be fetched afterwards
      if (tunnelState.details && tunnelState.details.location) {
        this.setLocation(tunnelState.details.location);
      }
    } else if (tunnelState.state === 'disconnected') {
      // It may take some time to fetch the new user location.
      // So take the user to the last known location when disconnected.
      if (this.lastDisconnectedLocation) {
        this.setLocation(this.lastDisconnectedLocation);
      }
    }

    if (tunnelState.state === 'connected' || tunnelState.state === 'disconnected') {
      try {
        // Fetch the new user location
        const getLocationPromise = (this.getLocationPromise = this.daemonRpc.getLocation());
        const location = await getLocationPromise;
        // If the location is currently unavailable, do nothing! This only ever
        // happens when a custom relay is set or we are in a blocked state.
        if (!location) {
          return;
        }

        // Cache the user location
        // Note: hostname is only set for relay servers.
        if (location.hostname === null) {
          this.lastDisconnectedLocation = location;
        }

        // Broadcast the new location if it is the result of the most recent call to getLocation.
        if (getLocationPromise === this.getLocationPromise) {
          this.setLocation(location);
        }
      } catch (error) {
        log.error(`Failed to update the location: ${error.message}`);
      }
    }
  }

  private trayIconType(tunnelState: TunnelState, blockWhenDisconnected: boolean): TrayIconType {
    switch (tunnelState.state) {
      case 'connected':
        return 'secured';

      case 'connecting':
        return 'securing';

      case 'error':
        if (!tunnelState.details.blockFailure) {
          return 'securing';
        } else {
          return 'unsecured';
        }
      case 'disconnecting':
        return 'securing';

      case 'disconnected':
        if (blockWhenDisconnected) {
          return 'securing';
        } else {
          return 'unsecured';
        }
    }
  }

  private updateTrayIcon(tunnelState: TunnelState, blockWhenDisconnected: boolean) {
    const type = this.trayIconType(tunnelState, blockWhenDisconnected);

    if (this.trayIconController) {
      this.trayIconController.animateToIcon(type);
    }
  }

  private registerWindowListener(windowController: WindowController) {
    windowController.window?.on('focus', () => {
      // cancel notifications when window appears
      this.notificationController.cancelPendingNotifications();

      if (
        !this.accountData ||
        closeToExpiry(this.accountData.expiry, 4) ||
        hasExpired(this.accountData.expiry)
      ) {
        this.updateAccountData();
      }
    });

    windowController.window?.on('blur', () => {
      // ensure notification guard is reset
      this.notificationController.resetTunnelStateAnnouncements();
    });
  }

  private registerIpcListeners() {
    IpcMainEventChannel.state.handleGet(() => ({
      isConnected: this.connectedToDaemon,
      autoStart: getOpenAtLogin(),
      accountData: this.accountData,
      accountHistory: this.accountHistory,
      tunnelState: this.tunnelState,
      settings: this.settings,
      location: this.location,
      relayListPair: {
        relays: this.processRelaysForPresentation(
          this.relays,
          this.settings.relaySettings,
          this.settings.bridgeState,
        ),
        bridges: this.processBridgesForPresentation(this.relays, this.settings.bridgeState),
      },
      currentVersion: this.currentVersion,
      upgradeVersion: this.upgradeVersion,
      guiSettings: this.guiSettings.state,
      wireguardPublicKey: this.wireguardPublicKey,
      translations: this.translations,
      windowsSplitTunnelingApplications: this.windowsSplitTunnelingApplications,
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

    IpcMainEventChannel.tunnel.handleConnect(() => this.daemonRpc.connectTunnel());
    IpcMainEventChannel.tunnel.handleDisconnect(() => this.daemonRpc.disconnectTunnel());
    IpcMainEventChannel.tunnel.handleReconnect(() => this.daemonRpc.reconnectTunnel());

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
      const currentAccountToken = this.settings.accountToken;
      const response = await this.daemonRpc.submitVoucher(voucherCode);

      if (currentAccountToken) {
        this.accountDataCache.handleVoucherResponse(currentAccountToken, response);
      }

      return response;
    });
    IpcMainEventChannel.account.handleUpdateData(() => this.updateAccountData());

    IpcMainEventChannel.accountHistory.handleClear(async () => {
      await this.daemonRpc.clearAccountHistory();
      void this.updateAccountHistory();
    });

    IpcMainEventChannel.wireguardKeys.handleGenerateKey(async () => {
      try {
        return await this.daemonRpc.generateWireguardKey();
      } catch {
        return 'generation_failure';
      }
    });
    IpcMainEventChannel.wireguardKeys.handleVerifyKey(() => this.daemonRpc.verifyWireguardKey());

    IpcMainEventChannel.linuxSplitTunneling.handleGetApplications(() => {
      if (linuxSplitTunneling) {
        return linuxSplitTunneling.getApplications(this.locale);
      } else {
        throw Error('linuxSplitTunneling.getApplications function called without being imported');
      }
    });
    IpcMainEventChannel.windowsSplitTunneling.handleGetApplications((updateCaches: boolean) => {
      if (windowsSplitTunneling) {
        return windowsSplitTunneling.getApplications({
          updateCaches,
        });
      } else {
        throw Error('windowsSplitTunneling.getApplications function called without being imported');
      }
    });
    IpcMainEventChannel.linuxSplitTunneling.handleLaunchApplication((application) => {
      if (linuxSplitTunneling) {
        return linuxSplitTunneling.launchApplication(application);
      } else {
        throw Error('linuxSplitTunneling.launchApplication function called without being imported');
      }
    });

    IpcMainEventChannel.windowsSplitTunneling.handleSetState((enabled) => {
      if (windowsSplitTunneling) {
        return this.daemonRpc.setSplitTunnelingState(enabled);
      } else {
        throw Error('windowsSplitTunneling.setState function called without being imported');
      }
    });
    IpcMainEventChannel.windowsSplitTunneling.handleAddApplication(async (application) => {
      if (windowsSplitTunneling) {
        // If the applications is a string (path) it's an application picked with the file picker
        // that we want to add to the list of additional applications.
        if (typeof application === 'string') {
          this.guiSettings.addBrowsedForSplitTunnelingApplications(application);
          const applicationPath = await windowsSplitTunneling.addApplicationPathToCache(
            application,
          );
          await this.daemonRpc.addSplitTunnelingApplication(applicationPath);
        } else {
          await this.daemonRpc.addSplitTunnelingApplication(application.absolutepath);
        }
      } else {
        throw Error(
          'windowsSplitTunneling.handleAddApplication function called without being imported',
        );
      }
    });
    IpcMainEventChannel.windowsSplitTunneling.handleRemoveApplication((application) => {
      if (windowsSplitTunneling) {
        return this.daemonRpc.removeSplitTunnelingApplication(
          typeof application === 'string' ? application : application.absolutepath,
        );
      } else {
        throw Error(
          'windowsSplitTunneling.handleRemoveApplication function called without being imported',
        );
      }
    });

    IpcMainEventChannel.problemReport.handleCollectLogs((toRedact) => {
      const id = uuid.v4();
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
            log.debug(`Problem report was written to ${reportPath}`);
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

    if (windowsSplitTunneling) {
      this.guiSettings.browsedForSplitTunnelingApplications.forEach(
        windowsSplitTunneling.addApplicationPathToCache,
      );
    }
  }

  private async createNewAccount(): Promise<string> {
    try {
      return await this.daemonRpc.createNewAccount();
    } catch (error) {
      log.error(`Failed to create account: ${error.message}`);
      throw error;
    }
  }

  private async login(accountToken: AccountToken): Promise<void> {
    try {
      const verification = await this.verifyAccount(accountToken);

      if (verification.status === 'deferred') {
        log.warn(`Failed to get account data, logging in anyway: ${verification.error.message}`);
      }

      this.autoConnectOnWireguardKeyEvent = this.guiSettings.autoConnect;
      await this.daemonRpc.setAccount(accountToken);

      // Fallback if daemon doesn't send event.
      if (this.autoConnectOnWireguardKeyEvent) {
        this.autoConnectFallbackScheduler.schedule(
          () => this.wireguardKeygenEventAutoConnect(),
          AUTO_CONNECT_FALLBACK_DELAY,
        );
      }
    } catch (error) {
      log.error(`Failed to login: ${error.message}`);

      this.autoConnectOnWireguardKeyEvent = false;

      if (error instanceof InvalidAccountError) {
        throw Error(messages.gettext('Invalid account number'));
      } else {
        throw error;
      }
    }
  }

  private wireguardKeygenEventAutoConnect() {
    if (this.autoConnectOnWireguardKeyEvent) {
      this.autoConnectOnWireguardKeyEvent = false;
      this.autoConnectFallbackScheduler.cancel();
      void this.autoConnect();
    }
  }

  private async autoConnect() {
    if (!this.accountData || !hasExpired(this.accountData.expiry)) {
      try {
        log.info('Auto-connecting the tunnel');
        await this.daemonRpc.connectTunnel();
      } catch (error) {
        log.error(`Failed to auto-connect the tunnel: ${error.message}`);
      }
    }
  }

  private async logout(): Promise<void> {
    try {
      await this.daemonRpc.setAccount();

      this.autoConnectFallbackScheduler.cancel();
      this.accountExpiryNotificationScheduler.cancel();
    } catch (error) {
      log.info(`Failed to logout: ${error.message}`);

      throw error;
    }
  }

  private verifyAccount(accountToken: AccountToken): Promise<AccountVerification> {
    return new Promise((resolve, reject) => {
      this.accountDataCache.invalidate();
      this.accountDataCache.fetch(accountToken, {
        onFinish: () => resolve({ status: 'verified' }),
        onError: (error) => {
          if (error instanceof InvalidAccountError) {
            reject(error);
          } else {
            resolve({ status: 'deferred', error });
          }
        },
      });
    });
  }

  private updateAccountDataOnAccountChange(oldAccount?: string, newAccount?: string) {
    if (oldAccount && !newAccount) {
      this.accountDataCache.invalidate();
    } else if (!oldAccount && newAccount) {
      this.accountDataCache.fetch(newAccount);
    } else if (oldAccount && newAccount && oldAccount !== newAccount) {
      this.accountDataCache.fetch(newAccount);
    }
  }

  private updateAccountData() {
    if (this.connectedToDaemon && this.settings.accountToken) {
      this.accountDataCache.fetch(this.settings.accountToken);
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
    } catch (error) {
      log.error(`Failed to fetch the account history: ${error.message}`);
    }
  }

  private async fetchWireguardKey(): Promise<void> {
    try {
      this.setWireguardKey(await this.daemonRpc.getWireguardKey());
    } catch (error) {
      log.error(`Failed to fetch wireguard key: ${error.message}`);
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

      if (this.windowController) {
        IpcMainEventChannel.autoStart.notify(this.windowController.webContents, autoStart);
      }

      this.updateDaemonsAutoConnect();
    } catch (error) {
      log.error(
        `Failed to update the autostart to ${autoStart.toString()}. ${error.message.toString()}`,
      );
    }
    return Promise.resolve();
  }

  private async setUnpinnedWindow(unpinnedWindow: boolean): Promise<void> {
    this.guiSettings.unpinnedWindow = unpinnedWindow;

    if (this.tray && this.windowController) {
      this.tray.removeAllListeners();

      const window = await this.createWindow();
      this.windowController.replaceWindow(window, unpinnedWindow);

      await this.initializeWindow();
      this.windowController.show();
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
  }

  private blockPermissionRequests() {
    session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => {
      callback(false);
    });
    session.defaultSession.setPermissionCheckHandler(() => false);
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

  private blockNavigation() {
    app.on('web-contents-created', (_event, contents) => {
      contents.on('will-navigate', (event) => {
        event.preventDefault();
      });
    });
  }

  private async installDevTools() {
    const { default: installer, REACT_DEVELOPER_TOOLS, REDUX_DEVTOOLS } = await import(
      'electron-devtools-installer'
    );
    const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
    const options = { forceDownload, loadExtensionOptions: { allowFileAccess: true } };
    try {
      await installer(REACT_DEVELOPER_TOOLS, options);
      await installer(REDUX_DEVTOOLS, options);
    } catch (e) {
      log.info(`Error installing extension: ${e.message}`);
    }
  }

  // On both Linux and Windows the app height is applied incorrectly:
  // https://github.com/electron/electron/issues/28777
  private getContentHeight(): number {
    // The height we want achieve.
    const contentHeight = 568;

    switch (process.platform) {
      case 'darwin': {
        // The size of transparent area around arrow on macOS.
        const headerBarArrowHeight = 12;

        return this.guiSettings.unpinnedWindow
          ? contentHeight
          : contentHeight + headerBarArrowHeight;
      }
      case 'win32':
        // On Windows the app height ends up slightly lower than we set it to if running in unpinned
        // mode and the app becomes a tiny bit taller when pinned to task bar.
        return this.guiSettings.unpinnedWindow ? contentHeight + 19 : contentHeight - 1;
      case 'linux':
        // On Linux the app ends up slightly lower than we set it to.
        return contentHeight - 25;
      default:
        return contentHeight;
    }
  }

  private async createWindow(): Promise<BrowserWindow> {
    const height = this.getContentHeight();
    const width = 320;

    const options: Electron.BrowserWindowConstructorOptions = {
      useContentSize: true,
      width,
      minWidth: width,
      maxWidth: width,
      height,
      minHeight: height,
      maxHeight: height,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      show: false,
      frame: this.guiSettings.unpinnedWindow,
      transparent: !this.guiSettings.unpinnedWindow,
      webPreferences: {
        preload: path.join(__dirname, '../renderer/preloadBundle.js'),
        nodeIntegration: false,
        nodeIntegrationInWorker: false,
        nodeIntegrationInSubFrames: false,
        enableRemoteModule: false,
        sandbox: !SANDBOX_DISABLED,
        contextIsolation: true,
        spellcheck: false,
        devTools: process.env.NODE_ENV === 'development',
      },
    };

    switch (process.platform) {
      case 'darwin': {
        // setup window flags to mimic popover on macOS
        const appWindow = new BrowserWindow({
          ...options,
          titleBarStyle: this.guiSettings.unpinnedWindow ? 'default' : 'customButtonsOnHover',
          minimizable: this.guiSettings.unpinnedWindow,
          closable: this.guiSettings.unpinnedWindow,
        });

        // make the window visible on all workspaces and prevent the icon from showing in the dock
        // and app switcher.
        if (this.guiSettings.unpinnedWindow) {
          void app.dock.show();
        } else {
          appWindow.setVisibleOnAllWorkspaces(true);
          app.dock.hide();
        }

        return appWindow;
      }

      case 'win32': {
        // setup window flags to mimic an overlay window
        const appWindow = new BrowserWindow({
          ...options,
          // Due to a bug in Electron the app is sometimes placed behind other apps when opened.
          // Setting alwaysOnTop to true ensures that the app is placed on top. Electron issue:
          // https://github.com/electron/electron/issues/25915
          alwaysOnTop: !this.guiSettings.unpinnedWindow,
          skipTaskbar: !this.guiSettings.unpinnedWindow,
        });

        appWindow.removeMenu();

        return appWindow;
      }

      case 'linux':
        return new BrowserWindow({
          ...options,
          icon: await findIconPath('mullvad-vpn'),
        });

      default: {
        return new BrowserWindow(options);
      }
    }
  }

  // On macOS, hotkeys are bound to the app menu and won't work if it's not set,
  // even though the app menu itself is not visible because the app does not appear in the dock.
  private setMacOsAppMenu() {
    const mullvadVpnSubmenu: Electron.MenuItemConstructorOptions[] = [{ role: 'quit' }];
    if (process.env.NODE_ENV === 'development') {
      mullvadVpnSubmenu.unshift({ role: 'reload' }, { role: 'forceReload' });
    }

    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Mullvad VPN',
        submenu: mullvadVpnSubmenu,
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
          { type: 'separator' },
          { role: 'selectAll' },
        ],
      },
    ];
    Menu.setApplicationMenu(Menu.buildFromTemplate(template));
  }

  private setLinuxAppMenu() {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Mullvad VPN',
        submenu: [{ role: 'quit' }],
      },
    ];
    Menu.setApplicationMenu(Menu.buildFromTemplate(template));
  }

  private createTrayContextMenu() {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: sprintf(messages.pgettext('tray-icon-context-menu', 'Open %(mullvadVpn)s'), {
          mullvadVpn: messages.pgettext('generic', 'Mullvad VPN'),
        }),
        click: () => this.windowController?.show(),
      },
      { type: 'separator' },
      {
        label: this.getContextMenuActionButtonLabel(),
        click: () => {
          if (this.tunnelState.state === 'disconnected') {
            // Workaround: gRPC calls are sometimes delayed by a few seconds and setImmediate
            // mitigates this. https://github.com/grpc/grpc-node/issues/882
            setImmediate(() => void this.daemonRpc.connectTunnel());
          } else {
            setImmediate(() => void this.daemonRpc.disconnectTunnel());
          }
        },
      },
      {
        label: messages.gettext('Reconnect'),
        enabled: this.tunnelState.state === 'connected' || this.tunnelState.state === 'connecting',
        click: () => setImmediate(() => void this.daemonRpc.reconnectTunnel()),
      },
    ];

    return Menu.buildFromTemplate(template);
  }

  private getContextMenuActionButtonLabel() {
    switch (this.tunnelState.state) {
      case 'disconnected':
        return messages.gettext('Connect');
      case 'connecting':
        return messages.gettext('Cancel');
      case 'connected':
        return messages.gettext('Disconnect');
      case 'disconnecting':
        return '';
      case 'error':
        return this.tunnelState.details.blockFailure
          ? messages.gettext('Dismiss')
          : messages.gettext('Cancel');
    }
  }

  private addContextMenu(windowController: WindowController) {
    const menuTemplate: Electron.MenuItemConstructorOptions[] = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectAll' },
    ];

    // add inspect element on right click menu
    windowController.window?.webContents.on(
      'context-menu',
      (_e: Event, props: { x: number; y: number; isEditable: boolean }) => {
        const inspectTemplate = [
          {
            label: 'Inspect element',
            click() {
              windowController.window?.webContents.openDevTools({ mode: 'detach' });
              windowController.window?.webContents.inspectElement(props.x, props.y);
            },
          },
        ];

        if (props.isEditable) {
          // mixin 'inspect element' into standard menu when in development mode
          if (process.env.NODE_ENV === 'development') {
            const inputMenu: Electron.MenuItemConstructorOptions[] = [
              { type: 'separator' },
              ...inspectTemplate,
            ];

            Menu.buildFromTemplate(inputMenu).popup({ window: windowController.window });
          } else {
            Menu.buildFromTemplate(menuTemplate).popup({ window: windowController.window });
          }
        } else if (process.env.NODE_ENV === 'development') {
          // display inspect element for all non-editable
          // elements when in development mode
          Menu.buildFromTemplate(inspectTemplate).popup({ window: windowController.window });
        }
      },
    );
  }

  private createTray(): Tray {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setToolTip('Mullvad VPN');

    // disable double click on tray icon since it causes weird delay
    tray.setIgnoreDoubleClickEvents(true);

    return tray;
  }

  private installTrayClickHandlers() {
    if (this.guiSettings.unpinnedWindow) {
      if (process.platform === 'win32' || process.platform === 'darwin') {
        this.tray?.on('right-click', () =>
          // This needs to be executed on click since if it is added to the tray icon it will be
          // displayed on left click as well.
          this.tray?.popUpContextMenu(this.createTrayContextMenu()),
        );
      }
      this.tray?.on('click', () => this.windowController?.show());
    } else {
      this.tray?.on('click', () => {
        const isMacOsBigSur = process.platform === 'darwin' && parseInt(os.release(), 10) >= 20;
        if (isMacOsBigSur && !this.windowController?.isVisible()) {
          // This is a workaround for this Electron issue, when it's resolved
          // `this.windowController?.toggle()` should do the trick on all platforms:
          // https://github.com/electron/electron/issues/28776
          const contextMenu = Menu.buildFromTemplate([]);
          contextMenu.on('menu-will-show', () => this.windowController?.show());
          this.tray?.popUpContextMenu(contextMenu);
        } else {
          this.windowController?.toggle();
        }
      });
      this.tray?.on('right-click', () => this.windowController?.hide());
    }
  }

  private installWindowsMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    if (!this.guiSettings.unpinnedWindow) {
      windowController.window?.on('blur', () => {
        // Detect if blur happened when user had a cursor above the tray icon.
        const trayBounds = tray.getBounds();
        const cursorPos = screen.getCursorScreenPoint();
        const isCursorInside =
          cursorPos.x >= trayBounds.x &&
          cursorPos.y >= trayBounds.y &&
          cursorPos.x <= trayBounds.x + trayBounds.width &&
          cursorPos.y <= trayBounds.y + trayBounds.height;
        if (!isCursorInside && !this.browsingFiles) {
          windowController.hide();
        }
      });
    }
  }

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  private installMacOsMenubarAppWindowHandlers(windowController: WindowController) {
    if (!this.guiSettings.unpinnedWindow) {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
      const macEventMonitor = new NSEventMonitor();
      const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;

      windowController.window?.on('show', () =>
        macEventMonitor.start(eventMask, () => windowController.hide()),
      );
      windowController.window?.on('hide', () => macEventMonitor.stop());
      windowController.window?.on('blur', () => {
        // Make sure to hide the menubar window when other program captures the focus.
        // But avoid doing that when dev tools capture the focus to make it possible to inspect the UI
        if (
          windowController.window?.isVisible() &&
          !windowController.window?.webContents.isDevToolsFocused()
        ) {
          windowController.hide();
        }
      });
    }
  }

  private installWindowCloseHandler(windowController: WindowController) {
    if (this.guiSettings.unpinnedWindow) {
      windowController.window?.on('close', (closeEvent: Event) => {
        if (this.quitStage !== AppQuitStage.ready) {
          closeEvent.preventDefault();
          windowController.hide();
        }
      });
    }
  }

  private installGenericFocusHandlers(windowController: WindowController) {
    windowController.window?.on('focus', () => {
      IpcMainEventChannel.windowFocus.notify(windowController.webContents, true);
    });
    windowController.window?.on('blur', () => {
      IpcMainEventChannel.windowFocus.notify(windowController.webContents, false);
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

  private async openLink(url: string, withAuth?: boolean) {
    if (withAuth) {
      let token = '';
      try {
        token = await this.daemonRpc.getWwwAuthToken();
      } catch (e) {
        log.error(`Failed to get the WWW auth token: ${e.message}`);
      }
      return shell.openExternal(`${url}?token=${token}`);
    } else {
      return shell.openExternal(url);
    }
  }

  private getProblemReportPath(id: string): string {
    return path.join(app.getPath('temp'), `${id}.log`);
  }
}

const applicationMain = new ApplicationMain();
applicationMain.run();
