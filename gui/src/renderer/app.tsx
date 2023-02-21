import { batch, Provider } from 'react-redux';
import { Router } from 'react-router';
import { bindActionCreators } from 'redux';

import { ILinuxSplitTunnelingApplication, IWindowsApplication } from '../shared/application-types';
import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  DeviceEvent,
  DeviceState,
  IAccountData,
  IAppVersionInfo,
  IDevice,
  IDeviceRemoval,
  IDnsOptions,
  ILocation,
  IRelayListWithEndpointData,
  ISettings,
  liftConstraint,
  ObfuscationSettings,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import { IGuiSettingsState, SYSTEM_PREFERRED_LOCALE_KEY } from '../shared/gui-settings-state';
import { IChangelog, ICurrentAppVersionInfo, IHistoryObject } from '../shared/ipc-types';
import log, { ConsoleOutput } from '../shared/logging';
import { LogLevel } from '../shared/logging-types';
import { Scheduler } from '../shared/scheduler';
import AppRouter from './components/AppRouter';
import { Changelog } from './components/Changelog';
import ErrorBoundary from './components/ErrorBoundary';
import KeyboardNavigation from './components/KeyboardNavigation';
import Lang from './components/Lang';
import MacOsScrollbarDetection from './components/MacOsScrollbarDetection';
import { ModalContainer } from './components/Modal';
import { AppContext } from './context';
import History, { ITransitionSpecification, transitions } from './lib/history';
import { loadTranslations } from './lib/load-translations';
import IpcOutput from './lib/logging';
import { RoutePath } from './lib/routes';
import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import configureStore from './redux/store';
import userInterfaceActions from './redux/userinterface/actions';
import versionActions from './redux/version/actions';

const IpcRendererEventChannel = window.ipc;

interface IPreferredLocaleDescriptor {
  name: string;
  code: string;
}

type LoginState = 'none' | 'logging in' | 'creating account' | 'too many devices';

const SUPPORTED_LOCALE_LIST = [
  { name: 'Dansk', code: 'da' },
  { name: 'Deutsch', code: 'de' },
  { name: 'English', code: 'en' },
  { name: 'Español', code: 'es' },
  { name: 'Suomi', code: 'fi' },
  { name: 'Français', code: 'fr' },
  { name: 'Italiano', code: 'it' },
  { name: '日本語', code: 'ja' },
  { name: '한국어', code: 'ko' },
  { name: 'မြန်မာဘာသာ', code: 'my' },
  { name: 'Nederlands', code: 'nl' },
  { name: 'Norsk', code: 'nb' },
  { name: 'Polski', code: 'pl' },
  { name: 'Português', code: 'pt' },
  { name: 'Русский', code: 'ru' },
  { name: 'Svenska', code: 'sv' },
  { name: 'ภาษาไทย', code: 'th' },
  { name: 'Türkçe', code: 'tr' },
  { name: '简体中文', code: 'zh-CN' },
  { name: '繁體中文', code: 'zh-TW' },
];

export default class AppRenderer {
  private history: History;
  private reduxStore = configureStore();
  private reduxActions = {
    account: bindActionCreators(accountActions, this.reduxStore.dispatch),
    connection: bindActionCreators(connectionActions, this.reduxStore.dispatch),
    settings: bindActionCreators(settingsActions, this.reduxStore.dispatch),
    version: bindActionCreators(versionActions, this.reduxStore.dispatch),
    userInterface: bindActionCreators(userInterfaceActions, this.reduxStore.dispatch),
  };

  private location?: Partial<ILocation>;
  private lastDisconnectedLocation?: Partial<ILocation>;
  private relayList?: IRelayListWithEndpointData;
  private tunnelState!: TunnelState;
  private settings!: ISettings;
  private deviceState?: DeviceState;
  private loginState: LoginState = 'none';
  private previousLoginState: LoginState = 'none';
  private loginScheduler = new Scheduler();
  private connectedToDaemon = false;
  private getLocationPromise?: Promise<ILocation>;

  constructor() {
    log.addOutput(new ConsoleOutput(LogLevel.debug));
    log.addOutput(new IpcOutput(LogLevel.debug));

    IpcRendererEventChannel.window.listenShape((windowShapeParams) => {
      if (typeof windowShapeParams.arrowPosition === 'number') {
        this.reduxActions.userInterface.updateWindowArrowPosition(windowShapeParams.arrowPosition);
      }
    });

    IpcRendererEventChannel.daemon.listenConnected(() => {
      void this.onDaemonConnected();
    });

    IpcRendererEventChannel.daemon.listenDisconnected(() => {
      this.onDaemonDisconnected();
    });

    IpcRendererEventChannel.daemon.listenIsPerformingPostUpgrade((isPerformingPostUpgrade) => {
      this.setIsPerformingPostUpgrade(isPerformingPostUpgrade);
    });

    IpcRendererEventChannel.daemon.listenDaemonAllowed((daemonAllowed) => {
      this.reduxActions.userInterface.setDaemonAllowed(daemonAllowed);
    });

    IpcRendererEventChannel.account.listen((newAccountData?: IAccountData) => {
      this.setAccountExpiry(newAccountData?.expiry);
    });

    IpcRendererEventChannel.account.listenDevice((deviceEvent) => {
      this.handleDeviceEvent(deviceEvent);
    });

    IpcRendererEventChannel.account.listenDevices((devices) => {
      this.reduxActions.account.updateDevices(devices);
    });

    IpcRendererEventChannel.accountHistory.listen((newAccountHistory?: AccountToken) => {
      this.setAccountHistory(newAccountHistory);
    });

    IpcRendererEventChannel.tunnel.listen((newState: TunnelState) => {
      this.setTunnelState(newState);
      this.updateBlockedState(newState, this.settings.blockWhenDisconnected);
    });

    IpcRendererEventChannel.settings.listen((newSettings: ISettings) => {
      this.setSettings(newSettings);
      this.updateBlockedState(this.tunnelState, newSettings.blockWhenDisconnected);
    });

    IpcRendererEventChannel.relays.listen((relayListPair: IRelayListWithEndpointData) => {
      this.setRelayListPair(relayListPair);
    });

    IpcRendererEventChannel.currentVersion.listen((currentVersion: ICurrentAppVersionInfo) => {
      this.setCurrentVersion(currentVersion);
    });

    IpcRendererEventChannel.upgradeVersion.listen((upgradeVersion: IAppVersionInfo) => {
      this.setUpgradeVersion(upgradeVersion);
    });

    IpcRendererEventChannel.guiSettings.listen((guiSettings: IGuiSettingsState) => {
      this.setGuiSettings(guiSettings);
    });

    IpcRendererEventChannel.autoStart.listen((autoStart: boolean) => {
      this.storeAutoStart(autoStart);
    });

    IpcRendererEventChannel.windowsSplitTunneling.listen((applications: IWindowsApplication[]) => {
      this.reduxActions.settings.setSplitTunnelingApplications(applications);
    });

    IpcRendererEventChannel.window.listenFocus((focus: boolean) => {
      this.reduxActions.userInterface.setWindowFocused(focus);
    });

    IpcRendererEventChannel.window.listenMacOsScrollbarVisibility((visibility) => {
      this.reduxActions.userInterface.setMacOsScrollbarVisibility(visibility);
    });

    IpcRendererEventChannel.navigation.listenReset(() => this.history.pop(true));

    // Request the initial state from the main process
    const initialState = IpcRendererEventChannel.state.get();

    this.setLocale(initialState.translations.locale);
    loadTranslations(
      messages,
      initialState.translations.locale,
      initialState.translations.messages,
    );
    loadTranslations(
      relayLocations,
      initialState.translations.locale,
      initialState.translations.relayLocations,
    );

    this.setAccountExpiry(initialState.accountData?.expiry);
    this.setSettings(initialState.settings);
    this.setIsPerformingPostUpgrade(initialState.isPerformingPostUpgrade);

    if (initialState.daemonAllowed !== undefined) {
      this.reduxActions.userInterface.setDaemonAllowed(initialState.daemonAllowed);
    }

    if (initialState.deviceState) {
      const deviceState = initialState.deviceState;
      this.handleDeviceEvent(
        { type: deviceState.type, deviceState } as DeviceEvent,
        initialState.navigationHistory !== undefined,
      );
    }

    this.setAccountHistory(initialState.accountHistory);
    this.setTunnelState(initialState.tunnelState);
    this.updateBlockedState(initialState.tunnelState, initialState.settings.blockWhenDisconnected);

    this.setRelayListPair(initialState.relayList);
    this.setCurrentVersion(initialState.currentVersion);
    this.setUpgradeVersion(initialState.upgradeVersion);
    this.setGuiSettings(initialState.guiSettings);
    this.storeAutoStart(initialState.autoStart);
    this.setChangelog(initialState.changelog, initialState.forceShowChanges);

    if (initialState.macOsScrollbarVisibility !== undefined) {
      this.reduxActions.userInterface.setMacOsScrollbarVisibility(
        initialState.macOsScrollbarVisibility,
      );
    }

    if (initialState.isConnected) {
      void this.onDaemonConnected();
    }

    this.checkContentHeight(false);
    window.addEventListener('resize', () => {
      this.checkContentHeight(true);
    });

    if (initialState.windowsSplitTunnelingApplications) {
      this.reduxActions.settings.setSplitTunnelingApplications(
        initialState.windowsSplitTunnelingApplications,
      );
    }

    void this.updateLocation();

    if (initialState.navigationHistory) {
      // Set last action to POP to trigger automatic scrolling to saved coordinates.
      initialState.navigationHistory.lastAction = 'POP';
      this.history = History.fromSavedHistory(initialState.navigationHistory);
    } else {
      const navigationBase = this.getNavigationBase();
      this.history = new History(navigationBase);
    }

    if (window.env.e2e) {
      // Make the current location available to the tests if running e2e tests
      window.e2e = { location: this.history.location.pathname };
    }
  }

  public renderView() {
    return (
      <AppContext.Provider value={{ app: this }}>
        <Provider store={this.reduxStore}>
          <Lang>
            <Router history={this.history.asHistory}>
              <ErrorBoundary>
                <ModalContainer>
                  <KeyboardNavigation>
                    <AppRouter />
                    <Changelog />
                  </KeyboardNavigation>
                  {window.env.platform === 'darwin' && <MacOsScrollbarDetection />}
                </ModalContainer>
              </ErrorBoundary>
            </Router>
          </Lang>
        </Provider>
      </AppContext.Provider>
    );
  }

  public submitVoucher = (code: string) => IpcRendererEventChannel.account.submitVoucher(code);
  public updateAccountData = () => IpcRendererEventChannel.account.updateData();
  public getDeviceState = () => IpcRendererEventChannel.account.getDeviceState();
  public removeDevice = (device: IDeviceRemoval) =>
    IpcRendererEventChannel.account.removeDevice(device);
  public connectTunnel = () => IpcRendererEventChannel.tunnel.connect();
  public disconnectTunnel = () => IpcRendererEventChannel.tunnel.disconnect();
  public reconnectTunnel = () => IpcRendererEventChannel.tunnel.reconnect();
  public updateRelaySettings = (relaySettings: RelaySettingsUpdate) =>
    IpcRendererEventChannel.settings.updateRelaySettings(relaySettings);
  public updateBridgeSettings = (bridgeSettings: BridgeSettings) =>
    IpcRendererEventChannel.settings.updateBridgeSettings(bridgeSettings);
  public setDnsOptions = (dnsOptions: IDnsOptions) =>
    IpcRendererEventChannel.settings.setDnsOptions(dnsOptions);
  public clearAccountHistory = () => IpcRendererEventChannel.accountHistory.clear();
  public setAutoConnect = (value: boolean) =>
    IpcRendererEventChannel.guiSettings.setAutoConnect(value);
  public setEnableSystemNotifications = (value: boolean) =>
    IpcRendererEventChannel.guiSettings.setEnableSystemNotifications(value);
  public setStartMinimized = (value: boolean) =>
    IpcRendererEventChannel.guiSettings.setStartMinimized(value);
  public setMonochromaticIcon = (value: boolean) =>
    IpcRendererEventChannel.guiSettings.setMonochromaticIcon(value);
  public setUnpinnedWindow = (value: boolean) =>
    IpcRendererEventChannel.guiSettings.setUnpinnedWindow(value);
  public getLinuxSplitTunnelingApplications = () =>
    IpcRendererEventChannel.linuxSplitTunneling.getApplications();
  public launchExcludedApplication = (application: ILinuxSplitTunnelingApplication | string) =>
    IpcRendererEventChannel.linuxSplitTunneling.launchApplication(application);
  public setSplitTunnelingState = (state: boolean) =>
    IpcRendererEventChannel.windowsSplitTunneling.setState(state);
  public addSplitTunnelingApplication = (application: string | IWindowsApplication) =>
    IpcRendererEventChannel.windowsSplitTunneling.addApplication(application);
  public forgetManuallyAddedSplitTunnelingApplication = (application: IWindowsApplication) =>
    IpcRendererEventChannel.windowsSplitTunneling.forgetManuallyAddedApplication(application);
  public setObfuscationSettings = (obfuscationSettings: ObfuscationSettings) =>
    IpcRendererEventChannel.settings.setObfuscationSettings(obfuscationSettings);
  public collectProblemReport = (toRedact: string | undefined) =>
    IpcRendererEventChannel.problemReport.collectLogs(toRedact);
  public viewLog = (path: string) => IpcRendererEventChannel.problemReport.viewLog(path);
  public quit = () => IpcRendererEventChannel.app.quit();
  public openUrl = (url: string) => IpcRendererEventChannel.app.openUrl(url);
  public showOpenDialog = (options: Electron.OpenDialogOptions) =>
    IpcRendererEventChannel.app.showOpenDialog(options);

  public login = async (accountToken: AccountToken) => {
    const actions = this.reduxActions;
    actions.account.startLogin(accountToken);

    log.info('Logging in');

    this.previousLoginState = this.loginState;
    this.loginState = 'logging in';

    try {
      await IpcRendererEventChannel.account.login(accountToken);
    } catch (e) {
      const error = e as Error;
      if (error.message === 'Too many devices') {
        try {
          await this.fetchDevices(accountToken);

          actions.account.loginTooManyDevices(error);
          this.loginState = 'too many devices';

          this.history.reset(RoutePath.tooManyDevices, { transition: transitions.push });
        } catch (e) {
          const error = e as Error;
          log.error('Failed to fetch device list');
          actions.account.loginFailed(error);
        }
      } else {
        actions.account.loginFailed(error);
      }
    }
  };

  public cancelLogin = (): void => {
    const reduxAccount = this.reduxActions.account;
    reduxAccount.loggedOut();
    this.loginState = 'none';
  };

  public logout = async (transition = transitions.dismiss) => {
    try {
      this.history.reset(RoutePath.login, { transition });
      await IpcRendererEventChannel.account.logout();
    } catch (e) {
      const error = e as Error;
      log.info('Failed to logout: ', error.message);
    }
  };

  public leaveRevokedDevice = async () => {
    await this.logout(transitions.pop);
    await this.disconnectTunnel();
  };

  public async createNewAccount() {
    log.info('Creating account');

    const actions = this.reduxActions;
    actions.account.startCreateAccount();
    this.loginState = 'creating account';

    try {
      await IpcRendererEventChannel.account.create();
      this.redirectToConnect();
    } catch (e) {
      const error = e as Error;
      actions.account.createAccountFailed(error);
    }
  }

  public fetchDevices = async (accountToken: AccountToken): Promise<Array<IDevice>> => {
    const devices = await IpcRendererEventChannel.account.listDevices(accountToken);
    this.reduxActions.account.updateDevices(devices);
    return devices;
  };

  public openLinkWithAuth = async (link: string): Promise<void> => {
    let token = '';
    try {
      token = await IpcRendererEventChannel.account.getWwwAuthToken();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to get the WWW auth token: ${error.message}`);
    }
    void this.openUrl(`${link}?token=${token}`);
  };

  public setAllowLan = async (allowLan: boolean) => {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setAllowLan(allowLan);
    actions.settings.updateAllowLan(allowLan);
  };

  public setShowBetaReleases = async (showBetaReleases: boolean) => {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setShowBetaReleases(showBetaReleases);
    actions.settings.updateShowBetaReleases(showBetaReleases);
  };

  public setEnableIpv6 = async (enableIpv6: boolean) => {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setEnableIpv6(enableIpv6);
    actions.settings.updateEnableIpv6(enableIpv6);
  };

  public setBridgeState = async (bridgeState: BridgeState) => {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setBridgeState(bridgeState);
    actions.settings.updateBridgeState(bridgeState);
  };

  public setBlockWhenDisconnected = async (blockWhenDisconnected: boolean) => {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setBlockWhenDisconnected(blockWhenDisconnected);
    actions.settings.updateBlockWhenDisconnected(blockWhenDisconnected);
  };

  public setOpenVpnMssfix = async (mssfix?: number) => {
    const actions = this.reduxActions;
    actions.settings.updateOpenVpnMssfix(mssfix);
    await IpcRendererEventChannel.settings.setOpenVpnMssfix(mssfix);
  };

  public setWireguardMtu = async (mtu?: number) => {
    const actions = this.reduxActions;
    actions.settings.updateWireguardMtu(mtu);
    await IpcRendererEventChannel.settings.setWireguardMtu(mtu);
  };

  public setWireguardQuantumResistant = async (quantumResistant?: boolean) => {
    const actions = this.reduxActions;
    actions.settings.updateWireguardQuantumResistant(quantumResistant);
    await IpcRendererEventChannel.settings.setWireguardQuantumResistant(quantumResistant);
  };

  public setAutoStart = (autoStart: boolean): Promise<void> => {
    this.storeAutoStart(autoStart);

    return IpcRendererEventChannel.autoStart.set(autoStart);
  };

  public getWindowsSplitTunnelingApplications(updateCache = false) {
    return IpcRendererEventChannel.windowsSplitTunneling.getApplications(updateCache);
  }

  public removeSplitTunnelingApplication(application: IWindowsApplication) {
    void IpcRendererEventChannel.windowsSplitTunneling.removeApplication(application);
  }

  public async showLaunchDaemonSettings() {
    await IpcRendererEventChannel.app.showLaunchDaemonSettings();
  }

  public async sendProblemReport(
    email: string,
    message: string,
    savedReportId: string,
  ): Promise<void> {
    await IpcRendererEventChannel.problemReport.sendReport({ email, message, savedReportId });
  }

  public getPreferredLocaleList(): IPreferredLocaleDescriptor[] {
    return [
      {
        // TRANSLATORS: The option that represents the active operating system language in the
        // TRANSLATORS: user interface language selection list.
        name: messages.gettext('System default'),
        code: SYSTEM_PREFERRED_LOCALE_KEY,
      },
      ...SUPPORTED_LOCALE_LIST.sort((a, b) => a.name.localeCompare(b.name)),
    ];
  }

  public setPreferredLocale = async (preferredLocale: string): Promise<void> => {
    const translations = await IpcRendererEventChannel.guiSettings.setPreferredLocale(
      preferredLocale,
    );

    // set current locale
    this.setLocale(translations.locale);

    // load translations for new locale
    loadTranslations(messages, translations.locale, translations.messages);
    loadTranslations(relayLocations, translations.locale, translations.relayLocations);
  };

  public getPreferredLocaleDisplayName = (localeCode: string): string => {
    const preferredLocale = this.getPreferredLocaleList().find((item) => item.code === localeCode);

    return preferredLocale ? preferredLocale.name : '';
  };

  public setDisplayedChangelog = (): void => {
    IpcRendererEventChannel.currentVersion.displayedChangelog();
  };

  public setNavigationHistory(history: IHistoryObject) {
    IpcRendererEventChannel.navigation.setHistory(history);

    if (window.env.e2e) {
      window.e2e.location = history.entries[history.index].pathname;
    }
  }

  private isLoggedIn(): boolean {
    return this.deviceState?.type === 'logged in';
  }

  // Make sure that the content height is correct and log if it isn't. This is mostly for debugging
  // purposes since there's a bug in Electron that causes the app height to be another value than
  // the one we have set.
  // https://github.com/electron/electron/issues/28777
  private checkContentHeight(resize: boolean): void {
    const expectedContentHeight = 568;
    const contentHeight = window.innerHeight;
    if (contentHeight !== expectedContentHeight) {
      log.verbose(
        resize ? 'Resize:' : 'Initial:',
        `Wrong content height: ${contentHeight}, expected ${expectedContentHeight}`,
      );
    }
  }

  private redirectToConnect() {
    // Redirect the user after some time to allow for the 'Logged in' screen to be visible
    this.loginScheduler.schedule(() => this.resetNavigation(), 1000);
  }

  private setLocale(locale: string) {
    this.reduxActions.userInterface.updateLocale(locale);
  }

  private setRelaySettings(relaySettings: RelaySettings) {
    const actions = this.reduxActions;

    if ('normal' in relaySettings) {
      const {
        location,
        openvpnConstraints,
        wireguardConstraints,
        tunnelProtocol,
        providers,
        ownership,
      } = relaySettings.normal;

      actions.settings.updateRelay({
        normal: {
          location: liftConstraint(location),
          providers,
          ownership,
          openvpn: {
            port: liftConstraint(openvpnConstraints.port),
            protocol: liftConstraint(openvpnConstraints.protocol),
          },
          wireguard: {
            port: liftConstraint(wireguardConstraints.port),
            ipVersion: liftConstraint(wireguardConstraints.ipVersion),
            useMultihop: wireguardConstraints.useMultihop,
            entryLocation: liftConstraint(wireguardConstraints.entryLocation),
          },
          tunnelProtocol: liftConstraint(tunnelProtocol),
        },
      });
    } else if ('customTunnelEndpoint' in relaySettings) {
      const customTunnelEndpoint = relaySettings.customTunnelEndpoint;
      const config = customTunnelEndpoint.config;

      if ('openvpn' in config) {
        actions.settings.updateRelay({
          customTunnelEndpoint: {
            host: customTunnelEndpoint.host,
            port: config.openvpn.endpoint.port,
            protocol: config.openvpn.endpoint.protocol,
          },
        });
      } else if ('wireguard' in config) {
        // TODO: handle wireguard
      }
    }
  }

  private setBridgeSettings(bridgeSettings: BridgeSettings) {
    const actions = this.reduxActions;

    if ('normal' in bridgeSettings) {
      actions.settings.updateBridgeSettings({
        normal: {
          location: liftConstraint(bridgeSettings.normal.location),
        },
      });
    } else if ('custom' in bridgeSettings) {
      actions.settings.updateBridgeSettings({
        custom: bridgeSettings.custom,
      });
    }
  }

  private onDaemonConnected() {
    this.connectedToDaemon = true;
    this.reduxActions.userInterface.setConnectedToDaemon(true);
    this.reduxActions.userInterface.setDaemonAllowed(true);
    this.resetNavigation();
  }

  private onDaemonDisconnected() {
    this.connectedToDaemon = false;
    this.reduxActions.userInterface.setConnectedToDaemon(false);
    this.resetNavigation();
  }

  private resetNavigation() {
    if (this.history) {
      const pathname = this.history.location.pathname as RoutePath;
      const nextPath = this.getNavigationBase() as RoutePath;

      if (pathname !== nextPath) {
        // First level contains the possible next locations and the second level contains the
        // possible current locations.
        const navigationTransitions: Partial<
          Record<RoutePath, Partial<Record<RoutePath | '*', ITransitionSpecification>>>
        > = {
          [RoutePath.launch]: {
            [RoutePath.login]: transitions.pop,
            [RoutePath.main]: transitions.pop,
            '*': transitions.dismiss,
          },
          [RoutePath.login]: {
            [RoutePath.launch]: transitions.push,
            [RoutePath.main]: transitions.pop,
            [RoutePath.deviceRevoked]: transitions.pop,
            '*': transitions.dismiss,
          },
          [RoutePath.main]: {
            [RoutePath.launch]: transitions.push,
            [RoutePath.login]: transitions.push,
            [RoutePath.tooManyDevices]: transitions.push,
            '*': transitions.dismiss,
          },
          [RoutePath.deviceRevoked]: {
            '*': transitions.pop,
          },
        };

        const transition =
          navigationTransitions[nextPath]?.[pathname] ?? navigationTransitions[nextPath]?.['*'];
        this.history.reset(nextPath, { transition });
      }
    }
  }

  private getNavigationBase(): RoutePath {
    if (this.connectedToDaemon && this.deviceState !== undefined) {
      const loginState = this.reduxStore.getState().account.status;
      const deviceRevoked = loginState.type === 'none' && loginState.deviceRevoked;

      if (deviceRevoked) {
        return RoutePath.deviceRevoked;
      } else if (this.isLoggedIn()) {
        return RoutePath.main;
      } else {
        return RoutePath.login;
      }
    } else {
      return RoutePath.launch;
    }
  }

  private setAccountHistory(accountHistory?: AccountToken) {
    this.reduxActions.account.updateAccountHistory(accountHistory);
  }

  private setTunnelState(tunnelState: TunnelState) {
    const actions = this.reduxActions;

    log.verbose(`Tunnel state: ${tunnelState.state}`);

    this.tunnelState = tunnelState;

    batch(() => {
      switch (tunnelState.state) {
        case 'connecting':
          actions.connection.connecting(tunnelState.details);
          break;

        case 'connected':
          actions.connection.connected(tunnelState.details);
          break;

        case 'disconnecting':
          actions.connection.disconnecting(tunnelState.details);
          break;

        case 'disconnected':
          actions.connection.disconnected();
          break;

        case 'error':
          actions.connection.blocked(tunnelState.details);
          break;
      }

      // Update the location when entering a new tunnel state since it's likely changed.
      void this.updateLocation();
    });
  }

  private setSettings(newSettings: ISettings) {
    this.settings = newSettings;

    const reduxSettings = this.reduxActions.settings;

    reduxSettings.updateAllowLan(newSettings.allowLan);
    reduxSettings.updateEnableIpv6(newSettings.tunnelOptions.generic.enableIpv6);
    reduxSettings.updateBlockWhenDisconnected(newSettings.blockWhenDisconnected);
    reduxSettings.updateShowBetaReleases(newSettings.showBetaReleases);
    reduxSettings.updateOpenVpnMssfix(newSettings.tunnelOptions.openvpn.mssfix);
    reduxSettings.updateWireguardMtu(newSettings.tunnelOptions.wireguard.mtu);
    reduxSettings.updateWireguardQuantumResistant(
      newSettings.tunnelOptions.wireguard.quantumResistant,
    );
    reduxSettings.updateBridgeState(newSettings.bridgeState);
    reduxSettings.updateDnsOptions(newSettings.tunnelOptions.dns);
    reduxSettings.updateSplitTunnelingState(newSettings.splitTunnel.enableExclusions);
    reduxSettings.updateObfuscationSettings(newSettings.obfuscationSettings);

    this.setRelaySettings(newSettings.relaySettings);
    this.setBridgeSettings(newSettings.bridgeSettings);
  }

  private setIsPerformingPostUpgrade(isPerformingPostUpgrade: boolean) {
    this.reduxActions.userInterface.setIsPerformingPostUpgrade(isPerformingPostUpgrade);
  }

  private updateBlockedState(tunnelState: TunnelState, blockWhenDisconnected: boolean) {
    const actions = this.reduxActions.connection;
    switch (tunnelState.state) {
      case 'connecting':
        actions.updateBlockState(true);
        break;

      case 'connected':
        actions.updateBlockState(false);
        break;

      case 'disconnected':
        actions.updateBlockState(blockWhenDisconnected);
        break;

      case 'disconnecting':
        actions.updateBlockState(true);
        break;

      case 'error':
        actions.updateBlockState(!tunnelState.details.blockingError);
        break;
    }
  }

  private handleDeviceEvent(deviceEvent: DeviceEvent, preventRedirectToConnect?: boolean) {
    const reduxAccount = this.reduxActions.account;

    this.deviceState = deviceEvent.deviceState;

    switch (deviceEvent.type) {
      case 'logged in': {
        const accountToken = deviceEvent.deviceState.accountAndDevice.accountToken;
        const device = deviceEvent.deviceState.accountAndDevice.device;

        switch (this.loginState) {
          case 'none':
            reduxAccount.loggedIn(accountToken, device);
            this.resetNavigation();
            break;
          case 'logging in':
            reduxAccount.loggedIn(accountToken, device);

            if (this.previousLoginState === 'too many devices') {
              this.resetNavigation();
            } else if (!preventRedirectToConnect) {
              this.redirectToConnect();
            }
            break;
          case 'creating account':
            reduxAccount.accountCreated(accountToken, device, new Date().toISOString());
            break;
        }
        break;
      }
      case 'logged out':
        this.loginScheduler.cancel();
        reduxAccount.loggedOut();
        this.resetNavigation();
        break;
      case 'revoked': {
        this.loginScheduler.cancel();

        const oldAccountState = this.reduxStore.getState().account;
        if (oldAccountState.loggingOut) {
          reduxAccount.loggedOut();
        } else {
          reduxAccount.deviceRevoked();
        }

        this.resetNavigation();
        break;
      }
    }

    this.previousLoginState = this.loginState;
    this.loginState = 'none';
  }

  private setLocation(location: Partial<ILocation>) {
    this.location = location;
    this.propagateLocationToRedux();
  }

  private propagateLocationToRedux() {
    if (this.location) {
      this.reduxActions.connection.newLocation(this.location);
    }
  }

  private setRelayListPair(relayListPair?: IRelayListWithEndpointData) {
    this.relayList = relayListPair;
    this.propagateRelayListPairToRedux();
  }

  private propagateRelayListPairToRedux() {
    if (this.relayList) {
      this.reduxActions.settings.updateRelayLocations(this.relayList.relayList.countries);
      this.reduxActions.settings.updateWireguardEndpointData(this.relayList.wireguardEndpointData);
    }
  }

  private setCurrentVersion(versionInfo: ICurrentAppVersionInfo) {
    this.reduxActions.version.updateVersion(
      versionInfo.gui,
      versionInfo.isConsistent,
      versionInfo.isBeta,
    );
  }

  private setUpgradeVersion(upgradeVersion: IAppVersionInfo) {
    this.reduxActions.version.updateLatest(upgradeVersion);
  }

  private setGuiSettings(guiSettings: IGuiSettingsState) {
    this.reduxActions.settings.updateGuiSettings(guiSettings);
  }

  private setAccountExpiry(expiry?: string) {
    this.reduxActions.account.updateAccountExpiry(expiry);
  }

  private storeAutoStart(autoStart: boolean) {
    this.reduxActions.settings.updateAutoStart(autoStart);
  }

  private setChangelog(changelog: IChangelog, forceShowChanges: boolean) {
    this.reduxActions.userInterface.setChangelog(changelog);
    this.reduxActions.userInterface.setForceShowChanges(forceShowChanges);
  }

  private async updateLocation() {
    switch (this.tunnelState.state) {
      case 'disconnected': {
        if (this.lastDisconnectedLocation) {
          this.setLocation(this.lastDisconnectedLocation);
        }
        const location = await this.fetchLocation();
        if (location) {
          this.setLocation(location);
          this.lastDisconnectedLocation = location;
        }
        break;
      }
      case 'disconnecting':
        if (this.lastDisconnectedLocation) {
          this.setLocation(this.lastDisconnectedLocation);
        } else {
          // If there's no previous location while disconnecting we remove the location. We keep the
          // coordinates to prevent the map from jumping around.
          const { longitude, latitude } = this.reduxStore.getState().connection;
          this.setLocation({ longitude, latitude });
        }
        break;
      case 'connecting':
        this.setLocation(this.tunnelState.details?.location ?? this.getLocationFromConstraints());
        break;
      case 'connected': {
        if (this.tunnelState.details?.location) {
          this.setLocation(this.tunnelState.details.location);
        }
        const location = await this.fetchLocation();
        if (location) {
          this.setLocation(location);
        }
        break;
      }
    }
  }

  private async fetchLocation(): Promise<ILocation | void> {
    try {
      // Fetch the new user location
      const getLocationPromise = IpcRendererEventChannel.location.get();
      this.getLocationPromise = getLocationPromise;
      const location = await getLocationPromise;
      // If the location is currently unavailable, do nothing! This only ever happens when a
      // custom relay is set or we are in a blocked state.
      if (location && getLocationPromise === this.getLocationPromise) {
        return location;
      }
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to update the location: ${error.message}`);
    }
  }

  private getLocationFromConstraints(): Partial<ILocation> {
    const state = this.reduxStore.getState();
    const coordinates = {
      longitude: state.connection.longitude,
      latitude: state.connection.latitude,
    };

    const relaySettings = this.settings.relaySettings;
    if ('normal' in relaySettings) {
      const location = relaySettings.normal.location;
      if (location !== 'any' && 'only' in location) {
        const constraint = location.only;

        const relayLocations = state.settings.relayLocations;
        if ('country' in constraint) {
          const country = relayLocations.find(({ code }) => constraint.country === code);

          return { country: country?.name, ...coordinates };
        } else if ('city' in constraint) {
          const country = relayLocations.find(({ code }) => constraint.city[0] === code);
          const city = country?.cities.find(({ code }) => constraint.city[1] === code);

          return { country: country?.name, city: city?.name, ...coordinates };
        } else if ('hostname' in constraint) {
          const country = relayLocations.find(({ code }) => constraint.hostname[0] === code);
          const city = country?.cities.find((location) => location.code === constraint.hostname[1]);

          let entryHostname: string | undefined;
          const multihopConstraint = relaySettings.normal.wireguardConstraints.useMultihop;
          const entryLocationConstraint = relaySettings.normal.wireguardConstraints.entryLocation;
          if (
            multihopConstraint &&
            entryLocationConstraint !== 'any' &&
            'hostname' in entryLocationConstraint.only &&
            entryLocationConstraint.only.hostname.length === 3
          ) {
            entryHostname = entryLocationConstraint.only.hostname[2];
          }

          return {
            country: country?.name,
            city: city?.name,
            hostname: constraint.hostname[2],
            entryHostname,
            ...coordinates,
          };
        }
      }
    }

    return coordinates;
  }
}
