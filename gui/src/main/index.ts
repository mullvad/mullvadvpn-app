import { execFile } from 'child_process';
import { app, BrowserWindow, ipcMain, Menu, nativeImage, screen, Tray } from 'electron';
import log from 'electron-log';
import mkdirp from 'mkdirp';
import * as path from 'path';
import * as uuid from 'uuid';
import {
  AccountToken,
  BridgeState,
  DaemonEvent,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  IRelayListHostname,
  ISettings,
  KeygenEvent,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { loadTranslations, messages } from '../shared/gettext';
import { IpcMainEventChannel } from '../shared/ipc-event-channel';
import {
  backupLogFile,
  getLogsDirectory,
  getMainLogFile,
  getRendererLogFile,
  setupLogging,
} from '../shared/logging';
import AccountDataCache, { AccountFetchRetryAction } from './account-data-cache';
import { getOpenAtLogin, setOpenAtLogin } from './autostart';
import {
  ConnectionObserver,
  DaemonRpc,
  ResponseParseError,
  SubscriptionListener,
} from './daemon-rpc';
import { InvalidAccountError } from './errors';
import GuiSettings from './gui-settings';
import NotificationController from './notification-controller';
import { resolveBin } from './proc';
import ReconnectionBackoff from './reconnection-backoff';
import TrayIconController, { TrayIconType } from './tray-icon-controller';
import WindowController from './window-controller';

const VERSION_UPDATE_INTERVAL = 24 * 60 * 60 * 1000;

const DAEMON_RPC_PATH =
  process.platform === 'win32' ? '//./pipe/Mullvad VPN' : '/var/run/mullvad-vpn';

enum AppQuitStage {
  unready,
  initiated,
  ready,
}

export interface ICurrentAppVersionInfo {
  gui: string;
  daemon: string;
  isConsistent: boolean;
}

export interface IAppUpgradeInfo extends IAppVersionInfo {
  nextUpgrade?: string;
  upToDate: boolean;
}

type AccountVerification = { status: 'verified' } | { status: 'deferred'; error: Error };

class ApplicationMain {
  private notificationController = new NotificationController();
  private windowController?: WindowController;
  private trayIconController?: TrayIconController;

  private daemonRpc = new DaemonRpc();
  private reconnectBackoff = new ReconnectionBackoff();
  private connectedToDaemon = false;
  private quitStage = AppQuitStage.unready;

  private accountData?: IAccountData = undefined;
  private accountHistory: AccountToken[] = [];
  private tunnelState: TunnelState = { state: 'disconnected' };
  private settings: ISettings = {
    accountToken: undefined,
    allowLan: false,
    autoConnect: false,
    blockWhenDisconnected: false,
    relaySettings: {
      normal: {
        location: 'any',
        tunnelProtocol: 'any',
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
      location: 'any',
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
        fwmark: undefined,
      },
    },
  };
  private guiSettings = new GuiSettings();
  private location?: ILocation;
  private lastDisconnectedLocation?: ILocation;

  private relays: IRelayList = { countries: [] };

  private currentVersion: ICurrentAppVersionInfo = {
    daemon: '',
    gui: '',
    isConsistent: true,
  };

  private upgradeVersion: IAppUpgradeInfo = {
    currentIsSupported: true,
    latestStable: '',
    latest: '',
    nextUpgrade: undefined,
    upToDate: true,
  };
  private latestVersionInterval?: NodeJS.Timeout;

  // The UI locale which is set once from onReady handler
  private locale = 'en';

  private wireguardPublicKey?: string;

  private accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this.daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      this.accountData = accountData;

      if (this.windowController) {
        IpcMainEventChannel.account.notify(this.windowController.webContents, accountData);
      }
    },
  );

  public run() {
    // Since electron's GPU blacklists are broken, GPU acceleration won't work on older distros
    if (process.platform === 'linux') {
      app.commandLine.appendSwitch('--disable-gpu');
    }

    this.overrideAppPaths();

    if (this.ensureSingleInstance()) {
      return;
    }

    this.initLogging();

    log.info(`Running version ${app.getVersion()}`);

    if (process.platform === 'win32') {
      app.setAppUserModelId('net.mullvad.vpn');
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
        app.setPath('userData', path.join(appDataDir, app.getName()));
      } else {
        throw new Error('Missing %LOCALAPPDATA% environment variable');
      }
    }
  }

  private initLogging() {
    const logDirectory = getLogsDirectory();
    const mainLogFile = getMainLogFile();
    const logFiles = [mainLogFile, getRendererLogFile()];

    if (process.env.NODE_ENV !== 'development') {
      // Ensure log directory exists
      mkdirp.sync(logDirectory);

      for (const logFile of logFiles) {
        backupLogFile(logFile);
      }
    }

    setupLogging(mainLogFile);
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

    // The window is not closable on macOS to be able to hide the titlebar and workaround
    // a shadow bug rendered above the invisible title bar. This also prevents the window from
    // closing normally, even programmatically. Therefore re-enable the close button just before
    // quitting the app.
    // Github issue: https://github.com/electron/electron/issues/15008
    if (process.platform === 'darwin' && this.windowController) {
      this.windowController.window.setClosable(true);
    }
  }

  private getLocaleWithOverride(): string {
    const localeOverride = process.env.MULLVAD_LOCALE;
    if (localeOverride) {
      const trimmedLocaleOverride = localeOverride.trim();
      if (trimmedLocaleOverride.length > 0) {
        return trimmedLocaleOverride;
      }
    }

    return app.getLocale();
  }

  private onReady = async () => {
    this.locale = this.getLocaleWithOverride();

    log.info(`Detected locale: ${this.locale}`);

    loadTranslations(this.locale, messages);

    this.daemonRpc.addConnectionObserver(
      new ConnectionObserver(this.onDaemonConnected, this.onDaemonDisconnected),
    );
    this.connectToDaemon();

    const window = this.createWindow();
    const tray = this.createTray();

    const windowController = new WindowController(window, tray);
    const trayIconController = new TrayIconController(
      tray,
      'unsecured',
      process.platform === 'darwin' && this.guiSettings.monochromaticIcon,
    );

    this.registerWindowListener(windowController);
    this.registerIpcListeners();
    this.addContextMenu(window);

    this.windowController = windowController;
    this.trayIconController = trayIconController;

    this.guiSettings.onChange = (newState, oldState) => {
      if (
        process.platform === 'darwin' &&
        oldState.monochromaticIcon !== newState.monochromaticIcon
      ) {
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

    if (process.env.NODE_ENV === 'development') {
      await this.installDevTools();
      window.webContents.openDevTools({ mode: 'detach' });
    }

    switch (process.platform) {
      case 'win32':
        this.installWindowsMenubarAppWindowHandlers(tray, windowController);
        break;
      case 'darwin':
        this.installMacOsMenubarAppWindowHandlers(tray, windowController);
        this.setMacOsAppMenu();
        break;
      case 'linux':
        this.installGenericMenubarAppWindowHandlers(tray, windowController);
        this.installLinuxWindowCloseHandler(windowController);
        break;
      default:
        this.installGenericMenubarAppWindowHandlers(tray, windowController);
        break;
    }

    if (this.shouldShowWindowOnStart() || process.env.NODE_ENV === 'development') {
      windowController.show();
    }

    window.loadFile(path.resolve(path.join(__dirname, '../renderer/index.html')));
  };

  private onDaemonConnected = async () => {
    this.connectedToDaemon = true;

    // subscribe to events
    try {
      await this.subscribeEvents();
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

    // fetch relays
    try {
      this.setRelays(await this.daemonRpc.getRelayLocations(), this.settings.relaySettings);
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
    this.fetchLatestVersion();

    // start periodic updates
    this.startLatestVersionPeriodicUpdates();

    // notify user about inconsistent version
    if (
      process.env.NODE_ENV !== 'development' &&
      !this.shouldSuppressNotifications(true) &&
      !this.currentVersion.isConsistent
    ) {
      this.notificationController.notifyInconsistentVersion();
    }

    // reset the reconnect backoff when connection established.
    this.reconnectBackoff.reset();

    // notify renderer
    if (this.windowController) {
      IpcMainEventChannel.daemonConnected.notify(this.windowController.webContents);
    }
  };

  private onDaemonDisconnected = (error?: Error) => {
    // make sure we were connected before to distinguish between a failed attempt to reconnect and
    // connection loss.
    const wasConnected = this.connectedToDaemon;

    if (wasConnected) {
      this.connectedToDaemon = false;

      // stop periodic updates
      this.stopLatestVersionPeriodicUpdates();

      // update the tray icon to indicate that the computer is not secure anymore
      this.updateTrayIcon({ state: 'disconnected' }, false);

      // notify renderer process
      if (this.windowController) {
        IpcMainEventChannel.daemonDisconnected.notify(
          this.windowController.webContents,
          error ? error.message : undefined,
        );
      }
    }

    // recover connection on error
    if (error) {
      if (wasConnected) {
        log.error(`Lost connection to daemon: ${error.message}`);
      } else {
        log.error(`Failed to connect to daemon: ${error.message}`);
      }

      this.reconnectToDaemon();
    } else {
      log.info('Disconnected from the daemon');
    }
  };

  private connectToDaemon() {
    this.daemonRpc.connect({ path: DAEMON_RPC_PATH });
  }

  private reconnectToDaemon() {
    this.reconnectBackoff.attempt(() => {
      this.connectToDaemon();
    });
  }

  private recoverFromBootstrapError(_error?: Error) {
    // Attempt to reconnect to daemon if the program fails to fetch settings, tunnel state or
    // subscribe for RPC events.
    this.daemonRpc.disconnect();

    this.reconnectToDaemon();
  }

  private async subscribeEvents(): Promise<void> {
    const daemonEventListener = new SubscriptionListener(
      (daemonEvent: DaemonEvent) => {
        if ('tunnelState' in daemonEvent) {
          this.setTunnelState(daemonEvent.tunnelState);
        } else if ('settings' in daemonEvent) {
          this.setSettings(daemonEvent.settings);
        } else if ('relayList' in daemonEvent) {
          this.setRelays(daemonEvent.relayList, this.settings.relaySettings);
        } else if ('wireguardKey' in daemonEvent) {
          this.handleWireguardKeygenEvent(daemonEvent.wireguardKey);
        }
      },
      (error: Error) => {
        log.error(`Cannot deserialize the daemon event: ${error.message}`);

        if (error instanceof ResponseParseError && error.validationError) {
          log.error(error.validationError.message);
        }
      },
    );

    return this.daemonRpc.subscribeDaemonEventListener(daemonEventListener);
  }

  private setAccountHistory(accountHistory: AccountToken[]) {
    this.accountHistory = accountHistory;

    if (this.windowController) {
      IpcMainEventChannel.accountHistory.notify(this.windowController.webContents, accountHistory);
    }
  }

  private setWireguardKey(wireguardKey?: string) {
    this.wireguardPublicKey = wireguardKey;
    if (this.windowController) {
      IpcMainEventChannel.wireguardKeys.notify(this.windowController.webContents, wireguardKey);
    }
  }

  private handleWireguardKeygenEvent(event: KeygenEvent) {
    switch (event) {
      case 'too_many_keys':
      case 'generation_failure':
        this.notificationController.notifyKeyGenerationFailed();
        this.wireguardPublicKey = undefined;
        break;
      default:
        this.wireguardPublicKey = event.newKey;
    }
    if (this.windowController) {
      IpcMainEventChannel.wireguardKeys.notifyKeygenEvent(this.windowController.webContents, event);
    }
  }

  private setTunnelState(newState: TunnelState) {
    this.tunnelState = newState;
    this.updateTrayIcon(newState, this.settings.blockWhenDisconnected);
    this.updateLocation();

    if (!this.shouldSuppressNotifications(false)) {
      this.notificationController.notifyTunnelState(newState);
    }

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
      this.updateAccountHistory();
      this.fetchWireguardKey();
    }

    if (this.windowController) {
      IpcMainEventChannel.settings.notify(this.windowController.webContents, newSettings);
    }

    // since settings can have the relay constraints changed, the relay
    // list should also be updated
    this.setRelays(this.relays, newSettings.relaySettings);
  }

  private setLocation(newLocation: ILocation) {
    this.location = newLocation;

    if (this.windowController) {
      IpcMainEventChannel.location.notify(this.windowController.webContents, newLocation);
    }
  }

  private setRelays(newRelayList: IRelayList, relaySettings: RelaySettings) {
    this.relays = newRelayList;
    const filteredRelays = this.processRelaysForPresentation(newRelayList, relaySettings);

    if (this.windowController) {
      IpcMainEventChannel.relays.notify(this.windowController.webContents, filteredRelays);
    }
  }

  //
  private processRelaysForPresentation(
    relayList: IRelayList,
    relaySettings: RelaySettings,
  ): IRelayList {
    // TODO: once wireguard is stable, by default we should only filter by
    // hasToHaveOpenvpn || hasToHaveWg, until then, only filter wireguard
    // relays if tunnel constraints specify wireguard tunnels.
    const hasOpenVpnTunnels = (relay: IRelayListHostname): boolean => {
      if (relay.tunnels) {
        return relay.tunnels.openvpn.length > 0;
      } else {
        return false;
      }
    };
    const hasWireguardTunnels = (relay: IRelayListHostname): boolean => {
      if (relay.tunnels) {
        return relay.tunnels.wireguard.length > 0;
      } else {
        return false;
      }
    };
    let fnHasWantedTunnels = hasOpenVpnTunnels;

    if ('normal' in relaySettings) {
      const tunnelConstraints = relaySettings.normal.tunnelProtocol;
      if (tunnelConstraints !== 'any' && 'wireguard' === tunnelConstraints.only) {
        fnHasWantedTunnels = hasWireguardTunnels;
      }
    }

    return {
      countries: relayList.countries.map((country) => {
        return {
          ...country,
          cities: country.cities
            .map((city) => {
              return {
                ...city,
                relays: city.relays.filter(fnHasWantedTunnels),
              };
            })
            .filter((city) => city.relays.length > 0),
        };
      }),
    };
  }

  private setDaemonVersion(daemonVersion: string) {
    const guiVersion = app.getVersion().replace('.0', '');
    const versionInfo = {
      daemon: daemonVersion,
      gui: guiVersion,
      isConsistent: daemonVersion === guiVersion,
    };

    this.currentVersion = versionInfo;

    // notify renderer
    if (this.windowController) {
      IpcMainEventChannel.currentVersion.notify(this.windowController.webContents, versionInfo);
    }
  }

  private setLatestVersion(latestVersionInfo: IAppVersionInfo) {
    function isBeta(version: string) {
      return version.includes('-');
    }

    function nextUpgrade(
      current: string,
      latest: string,
      latestStable: string,
    ): string | undefined {
      if (isBeta(current)) {
        return current === latest ? undefined : latest;
      } else {
        return current === latestStable ? undefined : latestStable;
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

    const currentVersionInfo = this.currentVersion;
    const latestVersion = latestVersionInfo.latest;
    const latestStableVersion = latestVersionInfo.latestStable;

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

    this.upgradeVersion = upgradeInfo;

    // notify user to update the app if it became unsupported
    if (
      process.env.NODE_ENV !== 'development' &&
      !this.shouldSuppressNotifications(true) &&
      currentVersionInfo.isConsistent &&
      !latestVersionInfo.currentIsSupported &&
      upgradeVersion
    ) {
      this.notificationController.notifyUnsupportedVersion(upgradeVersion);
    }

    if (this.windowController) {
      IpcMainEventChannel.upgradeVersion.notify(this.windowController.webContents, upgradeInfo);
    }
  }

  private async fetchLatestVersion() {
    try {
      this.setLatestVersion(await this.daemonRpc.getVersionInfo());
    } catch (error) {
      log.error(`Failed to request the version info: ${error.message}`);
    }
  }

  private startLatestVersionPeriodicUpdates() {
    const handler = () => {
      this.fetchLatestVersion();
    };
    this.latestVersionInterval = global.setInterval(handler, VERSION_UPDATE_INTERVAL);
  }

  private stopLatestVersionPeriodicUpdates() {
    if (this.latestVersionInterval) {
      clearInterval(this.latestVersionInterval);

      this.latestVersionInterval = undefined;
    }
  }

  private shouldSuppressNotifications(isCriticalNotification: boolean): boolean {
    const isVisible = this.windowController ? this.windowController.isVisible() : false;

    if (isCriticalNotification) {
      return isVisible;
    } else {
      return isVisible || !this.guiSettings.enableSystemNotifications;
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
        const location = await this.daemonRpc.getLocation();
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

        // Broadcast the new location.
        // There is a chance that the location is not stale if the tunnel state before the location
        // request is the same as after receiving the response.
        if (this.tunnelState.state === tunnelState.state) {
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

      case 'blocked':
        switch (tunnelState.details.reason) {
          case 'set_firewall_policy_error':
            return 'unsecured';
          default:
            return 'securing';
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
    windowController.window.on('show', () => {
      // cancel notifications when window appears
      this.notificationController.cancelPendingNotifications();

      this.updateAccountExpiryIfNeeded();
    });

    windowController.window.on('hide', () => {
      // ensure notification guard is reset
      this.notificationController.resetTunnelStateAnnouncements();
    });
  }

  private registerIpcListeners() {
    IpcMainEventChannel.state.handleGet(() => ({
      locale: this.locale,
      isConnected: this.connectedToDaemon,
      autoStart: getOpenAtLogin(),
      accountData: this.accountData,
      accountHistory: this.accountHistory,
      tunnelState: this.tunnelState,
      settings: this.settings,
      location: this.location,
      relays: this.processRelaysForPresentation(this.relays, this.settings.relaySettings),
      currentVersion: this.currentVersion,
      upgradeVersion: this.upgradeVersion,
      guiSettings: this.guiSettings.state,
      wireguardPublicKey: this.wireguardPublicKey,
    }));

    IpcMainEventChannel.settings.handleAllowLan((allowLan: boolean) =>
      this.daemonRpc.setAllowLan(allowLan),
    );
    IpcMainEventChannel.settings.handleEnableIpv6((enableIpv6: boolean) =>
      this.daemonRpc.setEnableIpv6(enableIpv6),
    );
    IpcMainEventChannel.settings.handleBlockWhenDisconnected((blockWhenDisconnected: boolean) =>
      this.daemonRpc.setBlockWhenDisconnected(blockWhenDisconnected),
    );
    IpcMainEventChannel.settings.handleBridgeState((bridgeState: BridgeState) =>
      this.daemonRpc.setBridgeState(bridgeState),
    );
    IpcMainEventChannel.settings.handleOpenVpnMssfix((mssfix?: number) =>
      this.daemonRpc.setOpenVpnMssfix(mssfix),
    );
    IpcMainEventChannel.settings.handleUpdateRelaySettings((update: RelaySettingsUpdate) =>
      this.daemonRpc.updateRelaySettings(update),
    );

    IpcMainEventChannel.autoStart.handleSet((autoStart: boolean) => {
      return this.setAutoStart(autoStart);
    });

    IpcMainEventChannel.tunnel.handleConnect(() => this.daemonRpc.connectTunnel());
    IpcMainEventChannel.tunnel.handleDisconnect(() => this.daemonRpc.disconnectTunnel());

    IpcMainEventChannel.guiSettings.handleEnableSystemNotifications((flag: boolean) => {
      this.guiSettings.enableSystemNotifications = flag;
    });

    IpcMainEventChannel.guiSettings.handleAutoConnect((autoConnect: boolean) => {
      this.guiSettings.autoConnect = autoConnect;
    });

    IpcMainEventChannel.guiSettings.handleStartMinimized((startMinimized: boolean) => {
      this.guiSettings.startMinimized = startMinimized;
    });

    IpcMainEventChannel.guiSettings.handleMonochromaticIcon((monochromaticIcon: boolean) => {
      this.guiSettings.monochromaticIcon = monochromaticIcon;
    });

    IpcMainEventChannel.account.handleLogin((token: AccountToken) => this.login(token));
    IpcMainEventChannel.account.handleLogout(() => this.logout());

    IpcMainEventChannel.accountHistory.handleRemoveItem(async (token: AccountToken) => {
      await this.daemonRpc.removeAccountFromHistory(token);
      this.updateAccountHistory();
    });

    IpcMainEventChannel.wireguardKeys.handleGenerateKey(async () => {
      try {
        return await this.daemonRpc.generateWireguardKey();
      } catch {
        return 'generation_failure';
      }
    });
    IpcMainEventChannel.wireguardKeys.handleVerifyKey(() => this.daemonRpc.verifyWireguardKey());

    ipcMain.on('show-window', () => {
      const windowController = this.windowController;
      if (windowController) {
        windowController.show();
      }
    });

    ipcMain.on('collect-logs', (event: Electron.Event, requestId: string, toRedact: string[]) => {
      const reportPath = path.join(app.getPath('temp'), uuid.v4() + '.log');
      const executable = resolveBin('problem-report');
      const args = ['collect', '--output', reportPath];
      if (toRedact.length > 0) {
        args.push('--redact', ...toRedact);
      }

      execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
        if (error) {
          log.error(
            `Failed to collect a problem report.
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
      (
        event: Electron.Event,
        requestId: string,
        email: string,
        message: string,
        savedReport: string,
      ) => {
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
  }

  private async login(accountToken: AccountToken): Promise<void> {
    try {
      const verification = await this.verifyAccount(accountToken);

      if (verification.status === 'deferred') {
        log.warn(`Failed to get account data, logging in anyway: ${verification.error.message}`);
      }

      await this.daemonRpc.setAccount(accountToken);
    } catch (error) {
      log.error(`Failed to login: ${error.message}`);

      throw error;
    }
  }

  private async logout(): Promise<void> {
    try {
      await this.daemonRpc.setAccount();
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
        onError: (error): AccountFetchRetryAction => {
          if (error instanceof InvalidAccountError) {
            reject(error);
            return AccountFetchRetryAction.stop;
          } else {
            resolve({ status: 'deferred', error });
            return AccountFetchRetryAction.retry;
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

  private updateAccountExpiryIfNeeded() {
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
      this.daemonRpc.setAutoConnect(daemonAutoConnect);
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

  private async installDevTools() {
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
  }

  private createWindow(): BrowserWindow {
    const contentHeight = 568;

    // the size of transparent area around arrow on macOS
    const headerBarArrowHeight = 12;

    const options: Electron.BrowserWindowConstructorOptions = {
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
          titleBarStyle: 'customButtonsOnHover',
          minimizable: false,
          closable: false,
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

      default: {
        const appWindow = new BrowserWindow({
          ...options,
          frame: true,
        });

        return appWindow;
      }
    }
  }

  // On macOS, hotkeys are bound to the app menu and won't work if it's not set,
  // even though the app menu itself is not visible because the app does not appear in the dock.
  private setMacOsAppMenu() {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Mullvad',
        submenu: [{ role: 'quit' }],
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
  }

  private addContextMenu(window: BrowserWindow) {
    const menuTemplate: Electron.MenuItemConstructorOptions[] = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectall' },
    ];

    // add inspect element on right click menu
    window.webContents.on(
      'context-menu',
      (_e: Event, props: { x: number; y: number; isEditable: boolean }) => {
        const inspectTemplate = [
          {
            label: 'Inspect element',
            click() {
              window.webContents.openDevTools({ mode: 'detach' });
              window.webContents.inspectElement(props.x, props.y);
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

            Menu.buildFromTemplate(inputMenu).popup({ window });
          } else {
            Menu.buildFromTemplate(menuTemplate).popup({ window });
          }
        } else if (process.env.NODE_ENV === 'development') {
          // display inspect element for all non-editable
          // elements when in development mode
          Menu.buildFromTemplate(inspectTemplate).popup({ window });
        }
      },
    );
  }

  private createTray(): Tray {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setToolTip('Mullvad VPN');

    // disable double click on tray icon since it causes weird delay
    tray.setIgnoreDoubleClickEvents(true);

    // disable icon highlight on macOS
    if (process.platform === 'darwin') {
      tray.setHighlightMode('never');
    }

    return tray;
  }

  private installWindowsMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
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
  }

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  private installMacOsMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    // $FlowFixMe: this module is only available on macOS
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    const macEventMonitor = new NSEventMonitor();
    // tslint:disable-next-line
    const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;
    const window = windowController.window;

    window.on('show', () => macEventMonitor.start(eventMask, () => windowController.hide()));
    window.on('hide', () => macEventMonitor.stop());
    tray.on('click', () => windowController.toggle());
  }

  private installGenericMenubarAppWindowHandlers(tray: Tray, windowController: WindowController) {
    tray.on('click', () => {
      windowController.toggle();
    });
  }

  private installLinuxWindowCloseHandler(windowController: WindowController) {
    windowController.window.on('close', (closeEvent: Event) => {
      if (this.quitStage !== AppQuitStage.ready) {
        closeEvent.preventDefault();
        windowController.hide();
      }
    });
  }

  private shouldShowWindowOnStart(): boolean {
    switch (process.platform) {
      case 'win32':
        return false;
      case 'darwin':
        return false;
      case 'linux':
        return !this.guiSettings.startMinimized;
      default:
        return true;
    }
  }
}

const applicationMain = new ApplicationMain();
applicationMain.run();
