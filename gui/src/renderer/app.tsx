import * as React from 'react';
import { Provider } from 'react-redux';
import { Router } from 'react-router';
import { bindActionCreators } from 'redux';

import ErrorBoundary from './components/ErrorBoundary';
import { AppContext } from './context';
import AppRoutes from './routes';

import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import { IRelayLocationRedux, IWgKey } from './redux/settings/reducers';
import configureStore from './redux/store';
import userInterfaceActions from './redux/userinterface/actions';
import versionActions from './redux/version/actions';

import { ICurrentAppVersionInfo } from '../shared/ipc-types';
import { IApplication, ILinuxSplitTunnelingApplication } from '../shared/application-types';
import { IGuiSettingsState, SYSTEM_PREFERRED_LOCALE_KEY } from '../shared/gui-settings-state';
import { messages, relayLocations } from '../shared/gettext';
import log, { ConsoleOutput } from '../shared/logging';
import { IRelayListPair, LaunchApplicationResult } from '../shared/ipc-schema';
import consumePromise from '../shared/promise';
import { Scheduler } from '../shared/scheduler';
import History, { transitions } from './lib/history';
import { loadTranslations } from './lib/load-translations';

import {
  AccountToken,
  BridgeSettings,
  BridgeState,
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
  VoucherResponse,
} from '../shared/daemon-rpc-types';
import { LogLevel } from '../shared/logging-types';
import IpcOutput from './lib/logging';

// This function wraps all IPC calls to catch errors and then rethrow them without the
// "Uncaught Error:" prefix that's added by Electron.
// This is a temporary workaround which won't be required after this Electron PR has been merged and
// released: https://github.com/electron/electron/pull/28346
function handleReponse(ipc: typeof window.ipc): typeof window.ipc {
  const wrappedIpc: Record<string, Record<string, unknown>> = {};

  Object.entries(ipc).forEach(([groupName, group]) => {
    wrappedIpc[groupName] = {} as typeof group;

    Object.entries(group).forEach(([fnName, fn]) => {
      wrappedIpc[groupName][fnName] = (...args: Parameters<typeof fn>) => {
        const response = fn(...args);
        if (response instanceof Promise) {
          return response.catch((error: Error) => {
            throw new Error(error.message?.replace(/^Uncaught Error: /, ''));
          });
        } else {
          return response;
        }
      };
    });
  });

  return wrappedIpc as typeof window.ipc;
}

const IpcRendererEventChannel = handleReponse(window.ipc);

interface IPreferredLocaleDescriptor {
  name: string;
  code: string;
}

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
  { name: 'Język polski', code: 'pl' },
  { name: 'Português', code: 'pt' },
  { name: 'Русский', code: 'ru' },
  { name: 'Svenska', code: 'sv' },
  { name: 'ภาษาไทย', code: 'th' },
  { name: 'Türkçe', code: 'tr' },
  { name: '简体中文', code: 'zh-CN' },
  { name: '繁體中文', code: 'zh-TW' },
];

export default class AppRenderer {
  private history = new History('/');
  private reduxStore = configureStore();
  private reduxActions = {
    account: bindActionCreators(accountActions, this.reduxStore.dispatch),
    connection: bindActionCreators(connectionActions, this.reduxStore.dispatch),
    settings: bindActionCreators(settingsActions, this.reduxStore.dispatch),
    version: bindActionCreators(versionActions, this.reduxStore.dispatch),
    userInterface: bindActionCreators(userInterfaceActions, this.reduxStore.dispatch),
  };

  private locale = 'en';
  private location?: ILocation;
  private relayListPair!: IRelayListPair;
  private tunnelState!: TunnelState;
  private settings!: ISettings;
  private guiSettings!: IGuiSettingsState;
  private autoConnected = false;
  private doingLogin = false;
  private loginScheduler = new Scheduler();
  private connectedToDaemon = false;

  constructor() {
    log.addOutput(new ConsoleOutput(LogLevel.debug));
    log.addOutput(new IpcOutput(LogLevel.debug));

    IpcRendererEventChannel.windowShape.listen((windowShapeParams) => {
      if (typeof windowShapeParams.arrowPosition === 'number') {
        this.reduxActions.userInterface.updateWindowArrowPosition(windowShapeParams.arrowPosition);
      }
    });

    IpcRendererEventChannel.daemon.listenConnected(() => {
      consumePromise(this.onDaemonConnected());
    });

    IpcRendererEventChannel.daemon.listenDisconnected(() => {
      this.onDaemonDisconnected();
    });

    IpcRendererEventChannel.account.listen((newAccountData?: IAccountData) => {
      this.setAccountExpiry(newAccountData?.expiry, newAccountData?.previousExpiry);
    });

    IpcRendererEventChannel.accountHistory.listen((newAccountHistory?: AccountToken) => {
      this.setAccountHistory(newAccountHistory);
    });

    IpcRendererEventChannel.tunnel.listen((newState: TunnelState) => {
      this.setTunnelState(newState);
      this.updateBlockedState(newState, this.settings.blockWhenDisconnected);
    });

    IpcRendererEventChannel.settings.listen((newSettings: ISettings) => {
      const oldSettings = this.settings;

      this.setSettings(newSettings);
      this.handleAccountChange(oldSettings.accountToken, newSettings.accountToken);
      this.updateBlockedState(this.tunnelState, newSettings.blockWhenDisconnected);
    });

    IpcRendererEventChannel.location.listen((newLocation: ILocation) => {
      this.setLocation(newLocation);
    });

    IpcRendererEventChannel.relays.listen((relayListPair: IRelayListPair) => {
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

    IpcRendererEventChannel.wireguardKeys.listenPublicKey((publicKey?: IWireguardPublicKey) => {
      this.setWireguardPublicKey(publicKey);
    });

    IpcRendererEventChannel.wireguardKeys.listenKeygenEvent((event: KeygenEvent) => {
      this.reduxActions.settings.setWireguardKeygenEvent(event);
    });

    IpcRendererEventChannel.windowsSplitTunneling.listen((applications: IApplication[]) => {
      this.reduxActions.settings.setSplitTunnelingApplications(applications);
    });

    IpcRendererEventChannel.windowFocus.listen((focus: boolean) => {
      this.reduxActions.userInterface.setWindowFocused(focus);
    });

    // Request the initial state from the main process
    const initialState = IpcRendererEventChannel.state.get();

    this.setLocale(initialState.locale);
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

    this.setAccountExpiry(
      initialState.accountData?.expiry,
      initialState.accountData?.previousExpiry,
    );
    this.handleAccountChange(undefined, initialState.settings.accountToken);
    this.setAccountHistory(initialState.accountHistory);
    this.setSettings(initialState.settings);
    this.setTunnelState(initialState.tunnelState);
    this.updateBlockedState(initialState.tunnelState, initialState.settings.blockWhenDisconnected);

    if (initialState.location) {
      this.setLocation(initialState.location);
    }

    this.setRelayListPair(initialState.relayListPair);
    this.setCurrentVersion(initialState.currentVersion);
    this.setUpgradeVersion(initialState.upgradeVersion);
    this.setGuiSettings(initialState.guiSettings);
    this.storeAutoStart(initialState.autoStart);
    this.setWireguardPublicKey(initialState.wireguardPublicKey);

    if (initialState.isConnected) {
      consumePromise(this.onDaemonConnected());
    }

    this.checkContentHeight();

    if (initialState.windowsSplitTunnelingApplications) {
      this.reduxActions.settings.setSplitTunnelingApplications(
        initialState.windowsSplitTunnelingApplications,
      );
    }
  }

  public renderView() {
    return (
      <AppContext.Provider value={{ app: this }}>
        <Provider store={this.reduxStore}>
          <Router history={this.history}>
            <ErrorBoundary>
              <AppRoutes />
            </ErrorBoundary>
          </Router>
        </Provider>
      </AppContext.Provider>
    );
  }

  public async login(accountToken: AccountToken) {
    const actions = this.reduxActions;
    actions.account.startLogin(accountToken);

    log.info('Logging in');

    this.doingLogin = true;

    try {
      await IpcRendererEventChannel.account.login(accountToken);
      actions.account.updateAccountToken(accountToken);
      actions.account.loggedIn();
      this.redirectToConnect();
    } catch (error) {
      actions.account.loginFailed(error);
    }
  }

  public async logout() {
    try {
      await IpcRendererEventChannel.account.logout();
    } catch (e) {
      log.info('Failed to logout: ', e.message);
    }
  }

  public async createNewAccount() {
    log.info('Creating account');

    const actions = this.reduxActions;
    actions.account.startCreateAccount();
    this.doingLogin = true;

    try {
      const accountToken = await IpcRendererEventChannel.account.create();
      const accountExpiry = new Date().toISOString();
      actions.account.accountCreated(accountToken, accountExpiry);
      this.redirectToConnect();
    } catch (error) {
      actions.account.createAccountFailed(error);
    }
  }

  public submitVoucher(voucherCode: string): Promise<VoucherResponse> {
    return IpcRendererEventChannel.account.submitVoucher(voucherCode);
  }

  public updateAccountData(): void {
    IpcRendererEventChannel.account.updateData();
  }

  public async connectTunnel(): Promise<void> {
    const state = this.tunnelState.state;

    // connect only if tunnel is disconnected or blocked.
    if (state === 'disconnecting' || state === 'disconnected' || state === 'error') {
      // switch to the connecting state ahead of time to make the app look more responsive
      this.reduxActions.connection.connecting();

      return IpcRendererEventChannel.tunnel.connect();
    }
  }

  public disconnectTunnel(): Promise<void> {
    return IpcRendererEventChannel.tunnel.disconnect();
  }

  public reconnectTunnel(): Promise<void> {
    return IpcRendererEventChannel.tunnel.reconnect();
  }

  public updateRelaySettings(relaySettings: RelaySettingsUpdate) {
    return IpcRendererEventChannel.settings.updateRelaySettings(relaySettings);
  }

  public updateBridgeSettings(bridgeSettings: BridgeSettings) {
    return IpcRendererEventChannel.settings.updateBridgeSettings(bridgeSettings);
  }

  public setDnsOptions(dns: IDnsOptions) {
    return IpcRendererEventChannel.settings.setDnsOptions(dns);
  }

  public clearAccountHistory(): Promise<void> {
    return IpcRendererEventChannel.accountHistory.clear();
  }

  public async openLinkWithAuth(link: string): Promise<void> {
    let token = '';
    try {
      token = await IpcRendererEventChannel.account.getWwwAuthToken();
    } catch (e) {
      log.error(`Failed to get the WWW auth token: ${e.message}`);
    }
    consumePromise(this.openUrl(`${link}?token=${token}`));
  }

  public async setAllowLan(allowLan: boolean) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setAllowLan(allowLan);
    actions.settings.updateAllowLan(allowLan);
  }

  public async setShowBetaReleases(showBetaReleases: boolean) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setShowBetaReleases(showBetaReleases);
    actions.settings.updateShowBetaReleases(showBetaReleases);
  }

  public async setEnableIpv6(enableIpv6: boolean) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setEnableIpv6(enableIpv6);
    actions.settings.updateEnableIpv6(enableIpv6);
  }

  public async setBridgeState(bridgeState: BridgeState) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setBridgeState(bridgeState);
    actions.settings.updateBridgeState(bridgeState);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setBlockWhenDisconnected(blockWhenDisconnected);
    actions.settings.updateBlockWhenDisconnected(blockWhenDisconnected);
  }

  public async setOpenVpnMssfix(mssfix?: number) {
    const actions = this.reduxActions;
    actions.settings.updateOpenVpnMssfix(mssfix);
    await IpcRendererEventChannel.settings.setOpenVpnMssfix(mssfix);
  }

  public async setWireguardMtu(mtu?: number) {
    const actions = this.reduxActions;
    actions.settings.updateWireguardMtu(mtu);
    await IpcRendererEventChannel.settings.setWireguardMtu(mtu);
  }

  public setAutoConnect(autoConnect: boolean) {
    IpcRendererEventChannel.guiSettings.setAutoConnect(autoConnect);
  }

  public setEnableSystemNotifications(flag: boolean) {
    IpcRendererEventChannel.guiSettings.setEnableSystemNotifications(flag);
  }

  public setAutoStart(autoStart: boolean): Promise<void> {
    this.storeAutoStart(autoStart);

    return IpcRendererEventChannel.autoStart.set(autoStart);
  }

  public setStartMinimized(startMinimized: boolean) {
    IpcRendererEventChannel.guiSettings.setStartMinimized(startMinimized);
  }

  public setMonochromaticIcon(monochromaticIcon: boolean) {
    IpcRendererEventChannel.guiSettings.setMonochromaticIcon(monochromaticIcon);
  }

  public setUnpinnedWindow(unpinnedWindow: boolean) {
    IpcRendererEventChannel.guiSettings.setUnpinnedWindow(unpinnedWindow);
  }

  public async verifyWireguardKey(publicKey: IWgKey) {
    const actions = this.reduxActions;
    actions.settings.verifyWireguardKey(publicKey);
    try {
      const valid = await IpcRendererEventChannel.wireguardKeys.verifyKey();
      actions.settings.completeWireguardKeyVerification(valid);
    } catch (error) {
      log.error(`Failed to verify WireGuard key - ${error.message}`);
      actions.settings.completeWireguardKeyVerification(undefined);
    }
  }

  public async generateWireguardKey() {
    const actions = this.reduxActions;
    actions.settings.generateWireguardKey();
    const keygenEvent = await IpcRendererEventChannel.wireguardKeys.generateKey();
    actions.settings.setWireguardKeygenEvent(keygenEvent);
  }

  public async replaceWireguardKey(oldKey: IWgKey) {
    const actions = this.reduxActions;
    actions.settings.replaceWireguardKey(oldKey);
    const keygenEvent = await IpcRendererEventChannel.wireguardKeys.generateKey();
    actions.settings.setWireguardKeygenEvent(keygenEvent);
  }

  public getLinuxSplitTunnelingApplications() {
    return IpcRendererEventChannel.linuxSplitTunneling.getApplications();
  }

  public getWindowsSplitTunnelingApplications(updateCache = false) {
    return IpcRendererEventChannel.windowsSplitTunneling.getApplications(updateCache);
  }

  public launchExcludedApplication(
    application: ILinuxSplitTunnelingApplication | string,
  ): Promise<LaunchApplicationResult> {
    return IpcRendererEventChannel.linuxSplitTunneling.launchApplication(application);
  }

  public setSplitTunnelingState(enabled: boolean): Promise<void> {
    return IpcRendererEventChannel.windowsSplitTunneling.setState(enabled);
  }

  public addSplitTunnelingApplication(application: IApplication | string): Promise<void> {
    return IpcRendererEventChannel.windowsSplitTunneling.addApplication(application);
  }

  public removeSplitTunnelingApplication(application: IApplication | string) {
    consumePromise(IpcRendererEventChannel.windowsSplitTunneling.removeApplication(application));
  }

  public collectProblemReport(toRedact?: string): Promise<string> {
    return IpcRendererEventChannel.problemReport.collectLogs(toRedact);
  }

  public async sendProblemReport(
    email: string,
    message: string,
    savedReportId: string,
  ): Promise<void> {
    await IpcRendererEventChannel.problemReport.sendReport({ email, message, savedReportId });
  }

  public viewLog(id: string): Promise<string> {
    return IpcRendererEventChannel.problemReport.viewLog(id);
  }

  public quit(): void {
    IpcRendererEventChannel.app.quit();
  }

  public openUrl(url: string): Promise<void> {
    return IpcRendererEventChannel.app.openUrl(url);
  }

  public showOpenDialog(
    options: Electron.OpenDialogOptions,
  ): Promise<Electron.OpenDialogReturnValue> {
    return IpcRendererEventChannel.app.showOpenDialog(options);
  }

  public getPreferredLocaleList(): IPreferredLocaleDescriptor[] {
    return [
      {
        // TRANSLATORS: The option that represents the active operating system language in the
        // TRANSLATORS: user interface language selection list.
        name: messages.gettext('System default'),
        code: SYSTEM_PREFERRED_LOCALE_KEY,
      },
      ...SUPPORTED_LOCALE_LIST,
    ];
  }

  public async setPreferredLocale(preferredLocale: string): Promise<void> {
    const translations = await IpcRendererEventChannel.guiSettings.setPreferredLocale(
      preferredLocale,
    );

    // set current locale
    this.setLocale(translations.locale);

    // load translations for new locale
    loadTranslations(messages, translations.locale, translations.messages);
    loadTranslations(relayLocations, translations.locale, translations.relayLocations);

    // refresh the relay list pair with the new translations
    this.propagateRelayListPairToRedux();

    // refresh the location with the new translations
    this.propagateLocationToRedux();
  }

  public getPreferredLocaleDisplayName(localeCode: string): string {
    const preferredLocale = this.getPreferredLocaleList().find((item) => item.code === localeCode);

    return preferredLocale ? preferredLocale.name : '';
  }

  // Make sure that the content height is correct and log if it isn't. This is mostly for debugging
  // purposes since there's a bug in Electron that causes the app height to be another value than
  // the one we have set.
  // https://github.com/electron/electron/issues/28777
  private checkContentHeight(): void {
    let expectedContentHeight = 568;

    // The app content is 12px taller on macOS to fit the top arrow.
    if (window.platform === 'darwin' && !this.guiSettings.unpinnedWindow) {
      expectedContentHeight += 12;
    }

    const contentHeight = window.innerHeight;
    if (contentHeight !== expectedContentHeight) {
      log.error(`Wrong content height ${contentHeight}, expected ${expectedContentHeight}`);
    }
  }

  private redirectToConnect() {
    // Redirect the user after some time to allow for the 'Logged in' screen to be visible
    this.loginScheduler.schedule(() => this.resetNavigation(), 1000);
  }

  private setLocale(locale: string) {
    this.locale = locale;
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
      } = relaySettings.normal;

      actions.settings.updateRelay({
        normal: {
          location: liftConstraint(location),
          openvpn: {
            port: liftConstraint(openvpnConstraints.port),
            protocol: liftConstraint(openvpnConstraints.protocol),
          },
          wireguard: { port: liftConstraint(wireguardConstraints.port) },
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

  private async onDaemonConnected() {
    this.connectedToDaemon = true;
    await this.autoConnect();
    this.resetNavigation();
  }

  private onDaemonDisconnected() {
    this.connectedToDaemon = false;
    this.resetNavigation();
  }

  private resetNavigation() {
    const pathname = this.history.location.pathname;

    if (this.connectedToDaemon) {
      if (this.settings.accountToken) {
        const transition =
          pathname === '/login' || pathname === '/' ? transitions.push : transitions.dismiss;
        this.history.reset('/main', transition);
      } else {
        const transition =
          pathname === '/main'
            ? transitions.pop
            : pathname === '/'
            ? transitions.push
            : transitions.none;

        this.history.reset('/login', transition);
      }
    } else {
      const transition =
        pathname === '/login' || pathname === '/main' ? transitions.pop : transitions.dismiss;
      this.history.reset('/', transition);
    }
  }

  private async autoConnect() {
    if (window.runningInDevelopment) {
      log.info('Skip autoconnect in development');
    } else if (this.autoConnected) {
      log.info('Skip autoconnect because it was done before');
    } else if (this.settings.accountToken) {
      if (this.guiSettings.autoConnect) {
        try {
          log.info('Autoconnect the tunnel');

          await this.connectTunnel();

          this.autoConnected = true;
        } catch (error) {
          log.error(`Failed to autoconnect the tunnel: ${error.message}`);
        }
      } else {
        log.info('Skip autoconnect because GUI setting is disabled');
      }
    } else {
      log.info('Skip autoconnect because account token is not set');
    }
  }

  private setAccountHistory(accountHistory?: AccountToken) {
    this.reduxActions.account.updateAccountHistory(accountHistory);
  }

  private setTunnelState(tunnelState: TunnelState) {
    const actions = this.reduxActions;

    log.debug(`Tunnel state: ${tunnelState.state}`);

    this.tunnelState = tunnelState;

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
    reduxSettings.updateBridgeState(newSettings.bridgeState);
    reduxSettings.updateDnsOptions(newSettings.tunnelOptions.dns);
    reduxSettings.updateSplitTunnelingState(newSettings.splitTunnel.enableExclusions);

    this.setRelaySettings(newSettings.relaySettings);
    this.setBridgeSettings(newSettings.bridgeSettings);
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
        actions.updateBlockState(!tunnelState.details.blockFailure);
        break;
    }
  }

  private handleAccountChange(oldAccount?: string, newAccount?: string) {
    const reduxAccount = this.reduxActions.account;

    if (oldAccount && !newAccount) {
      this.loginScheduler.cancel();
      reduxAccount.loggedOut();

      this.resetNavigation();
    } else if (newAccount && oldAccount !== newAccount && !this.doingLogin) {
      reduxAccount.updateAccountToken(newAccount);
      reduxAccount.loggedIn();

      this.resetNavigation();
    }

    this.doingLogin = false;
  }

  private setLocation(location: ILocation) {
    this.location = location;
    this.propagateLocationToRedux();
  }

  private propagateLocationToRedux() {
    if (this.location) {
      this.reduxActions.connection.newLocation(this.translateLocation(this.location));
    }
  }

  private translateLocation(inputLocation: ILocation): ILocation {
    const location = { ...inputLocation };

    if (location.city) {
      const city = location.city;

      location.city = relayLocations.gettext(city) || city;
    }

    if (location.country) {
      const country = location.country;

      location.country = relayLocations.gettext(country) || country;
    }

    return location;
  }

  private convertRelayListToLocationList(relayList: IRelayList): IRelayLocationRedux[] {
    return relayList.countries
      .map((country) => ({
        name: relayLocations.gettext(country.name) || country.name,
        code: country.code,
        hasActiveRelays: country.cities.some((city) => city.relays.some((relay) => relay.active)),
        cities: country.cities
          .map((city) => ({
            name: relayLocations.gettext(city.name) || city.name,
            code: city.code,
            latitude: city.latitude,
            longitude: city.longitude,
            hasActiveRelays: city.relays.some((relay) => relay.active),
            relays: city.relays.sort((relayA, relayB) =>
              relayA.hostname.localeCompare(relayB.hostname, this.locale, { numeric: true }),
            ),
          }))
          .sort((cityA, cityB) => cityA.name.localeCompare(cityB.name, this.locale)),
      }))
      .sort((countryA, countryB) => countryA.name.localeCompare(countryB.name, this.locale));
  }

  private setRelayListPair(relayListPair: IRelayListPair) {
    this.relayListPair = relayListPair;
    this.propagateRelayListPairToRedux();
  }

  private propagateRelayListPairToRedux() {
    const relays = this.convertRelayListToLocationList(this.relayListPair.relays);
    const bridges = this.convertRelayListToLocationList(this.relayListPair.bridges);

    this.reduxActions.settings.updateRelayLocations(relays);
    this.reduxActions.settings.updateBridgeLocations(bridges);
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
    this.guiSettings = guiSettings;
    this.reduxActions.settings.updateGuiSettings(guiSettings);
  }

  private setAccountExpiry(expiry?: string, previousExpiry?: string) {
    this.reduxActions.account.updateAccountExpiry(expiry, previousExpiry);
  }

  private storeAutoStart(autoStart: boolean) {
    this.reduxActions.settings.updateAutoStart(autoStart);
  }

  private setWireguardPublicKey(publicKey?: IWireguardPublicKey) {
    this.reduxActions.settings.setWireguardKey(publicKey);
  }
}
