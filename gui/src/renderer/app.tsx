import {
  ConnectedRouter,
  push as pushHistory,
  replace as replaceHistory,
} from 'connected-react-router';
import { ipcRenderer, webFrame } from 'electron';
import log from 'electron-log';
import { createMemoryHistory } from 'history';
import * as React from 'react';
import { Provider } from 'react-redux';
import { bindActionCreators } from 'redux';

import ErrorBoundary from './components/ErrorBoundary';
import AppRoutes from './routes';

import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import configureStore from './redux/store';
import userInterfaceActions from './redux/userinterface/actions';
import versionActions from './redux/version/actions';

import { IAppUpgradeInfo, ICurrentAppVersionInfo } from '../main';
import { cities, countries, loadTranslations, messages, relayLocations } from '../shared/gettext';
import { IGuiSettingsState } from '../shared/gui-settings-state';
import { IpcRendererEventChannel } from '../shared/ipc-event-channel';
import { getRendererLogFile, setupLogging } from '../shared/logging';

import {
  AccountToken,
  BridgeState,
  IAccountData,
  ILocation,
  IRelayList,
  ISettings,
  KeygenEvent,
  liftConstraint,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelState,
} from '../shared/daemon-rpc-types';

export default class AppRenderer {
  private memoryHistory = createMemoryHistory();
  private reduxStore = configureStore(this.memoryHistory);
  private reduxActions = {
    account: bindActionCreators(accountActions, this.reduxStore.dispatch),
    connection: bindActionCreators(connectionActions, this.reduxStore.dispatch),
    settings: bindActionCreators(settingsActions, this.reduxStore.dispatch),
    version: bindActionCreators(versionActions, this.reduxStore.dispatch),
    userInterface: bindActionCreators(userInterfaceActions, this.reduxStore.dispatch),
    history: bindActionCreators(
      {
        push: pushHistory,
        replace: replaceHistory,
      },
      this.reduxStore.dispatch,
    ),
  };

  private locale: string;
  private tunnelState: TunnelState;
  private settings: ISettings;
  private guiSettings: IGuiSettingsState;
  private connectedToDaemon = false;
  private autoConnected = false;
  private doingLogin = false;
  private loginTimer?: NodeJS.Timeout;

  constructor() {
    setupLogging(getRendererLogFile());

    IpcRendererEventChannel.windowShape.listen((windowShapeParams) => {
      if (typeof windowShapeParams.arrowPosition === 'number') {
        this.reduxActions.userInterface.updateWindowArrowPosition(windowShapeParams.arrowPosition);
      }
    });

    IpcRendererEventChannel.daemonConnected.listen(() => {
      this.onDaemonConnected();
    });

    IpcRendererEventChannel.daemonDisconnected.listen((errorMessage?: string) => {
      this.onDaemonDisconnected(errorMessage ? new Error(errorMessage) : undefined);
    });

    IpcRendererEventChannel.account.listen((newAccountData?: IAccountData) => {
      this.setAccountExpiry(newAccountData && newAccountData.expiry);
    });

    IpcRendererEventChannel.accountHistory.listen((newAccountHistory: AccountToken[]) => {
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

    IpcRendererEventChannel.relays.listen((newRelays: IRelayList) => {
      this.setRelays(newRelays);
    });

    IpcRendererEventChannel.currentVersion.listen((currentVersion: ICurrentAppVersionInfo) => {
      this.setCurrentVersion(currentVersion);
    });

    IpcRendererEventChannel.upgradeVersion.listen((upgradeVersion: IAppUpgradeInfo) => {
      this.setUpgradeVersion(upgradeVersion);
    });

    IpcRendererEventChannel.guiSettings.listen((guiSettings: IGuiSettingsState) => {
      this.setGuiSettings(guiSettings);
    });

    IpcRendererEventChannel.autoStart.listen((autoStart: boolean) => {
      this.storeAutoStart(autoStart);
    });

    IpcRendererEventChannel.wireguardKeys.listen((publicKey?: string) => {
      this.setWireguardPublicKey(publicKey);
    });

    IpcRendererEventChannel.wireguardKeys.listenKeygenEvents((event: KeygenEvent) => {
      this.reduxActions.settings.setWireguardKeygenEvent(event);
    });

    // Request the initial state from the main process
    const initialState = IpcRendererEventChannel.state.get();

    this.locale = initialState.locale;
    this.tunnelState = initialState.tunnelState;
    this.settings = initialState.settings;
    this.guiSettings = initialState.guiSettings;

    this.setAccountExpiry(initialState.accountData && initialState.accountData.expiry);
    this.setAccountHistory(initialState.accountHistory);
    this.setSettings(initialState.settings);
    this.setTunnelState(initialState.tunnelState);
    this.updateBlockedState(initialState.tunnelState, initialState.settings.blockWhenDisconnected);

    if (initialState.location) {
      this.setLocation(initialState.location);
    }

    this.setRelays(initialState.relays);
    this.setCurrentVersion(initialState.currentVersion);
    this.setUpgradeVersion(initialState.upgradeVersion);
    this.setGuiSettings(initialState.guiSettings);
    this.storeAutoStart(initialState.autoStart);
    this.setWireguardPublicKey(initialState.wireguardPublicKey);

    if (initialState.isConnected) {
      this.onDaemonConnected();
    }

    // disable pinch to zoom
    webFrame.setVisualZoomLevelLimits(1, 1);

    // Load translations
    for (const catalogue of [messages, countries, cities, relayLocations]) {
      loadTranslations(this.locale, catalogue);
    }
  }

  public renderView() {
    return (
      <Provider store={this.reduxStore}>
        <ConnectedRouter history={this.memoryHistory}>
          <ErrorBoundary>
            <AppRoutes
              sharedProps={{
                app: this,
                locale: this.locale,
              }}
            />
          </ErrorBoundary>
        </ConnectedRouter>
      </Provider>
    );
  }

  public async login(accountToken: AccountToken) {
    const actions = this.reduxActions;
    actions.account.startLogin(accountToken);

    log.info('Logging in');

    this.doingLogin = true;

    try {
      await IpcRendererEventChannel.account.login(accountToken);

      // Redirect the user after some time to allow for the 'Logged in' screen to be visible
      this.loginTimer = global.setTimeout(async () => {
        this.memoryHistory.replace('/connect');

        try {
          log.info('Auto-connecting the tunnel');
          await this.connectTunnel();
        } catch (error) {
          log.error(`Failed to auto-connect the tunnel: ${error.message}`);
        }
      }, 1000);
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

  public async connectTunnel(): Promise<void> {
    const state = this.tunnelState.state;

    // connect only if tunnel is disconnected or blocked.
    if (state === 'disconnecting' || state === 'disconnected' || state === 'blocked') {
      // switch to the connecting state ahead of time to make the app look more responsive
      this.reduxActions.connection.connecting();

      return IpcRendererEventChannel.tunnel.connect();
    }
  }

  public disconnectTunnel(): Promise<void> {
    return IpcRendererEventChannel.tunnel.disconnect();
  }

  public updateRelaySettings(relaySettings: RelaySettingsUpdate) {
    return IpcRendererEventChannel.settings.updateRelaySettings(relaySettings);
  }

  public async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    return IpcRendererEventChannel.accountHistory.removeItem(accountToken);
  }

  public async setAllowLan(allowLan: boolean) {
    const actions = this.reduxActions;
    await IpcRendererEventChannel.settings.setAllowLan(allowLan);
    actions.settings.updateAllowLan(allowLan);
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

  public async verifyWireguardKey(publicKey: string) {
    const actions = this.reduxActions;
    actions.settings.verifyWireguardKey(publicKey);
    const valid = await IpcRendererEventChannel.wireguardKeys.verifyKey();
    actions.settings.completeWireguardKeyVerification(valid);
  }

  public async generateWireguardKey() {
    const actions = this.reduxActions;
    actions.settings.generateWireguardKey();
    const keygenEvent = await IpcRendererEventChannel.wireguardKeys.generateKey();
    actions.settings.setWireguardKeygenEvent(keygenEvent);
  }

  private setRelaySettings(relaySettings: RelaySettings) {
    const actions = this.reduxActions;

    if ('normal' in relaySettings) {
      const normal = relaySettings.normal;
      const tunnelProtocol = normal.tunnelProtocol;
      const location = normal.location;
      const relayLocation = location === 'any' ? 'any' : location.only;

      const { port, protocol } = normal.openvpnConstraints;
      const wireguardPort = normal.wireguardConstraints.port;
      actions.settings.updateRelay({
        normal: {
          location: relayLocation,
          openvpn: {
            port: port === 'any' ? port : port.only,
            protocol: protocol === 'any' ? protocol : protocol.only,
          },
          wireguard: { port: wireguardPort === 'any' ? wireguardPort : wireguardPort.only },
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

  private async onDaemonConnected() {
    this.connectedToDaemon = true;

    if (this.settings.accountToken) {
      this.memoryHistory.replace('/connect');

      // try to autoconnect the tunnel
      await this.autoConnect();
    } else {
      this.memoryHistory.replace('/login');

      // show window when account is not set
      ipcRenderer.send('show-window');
    }
  }

  private onDaemonDisconnected(error?: Error) {
    const wasConnected = this.connectedToDaemon;

    this.connectedToDaemon = false;

    if (error && wasConnected) {
      this.memoryHistory.replace('/');
    }
  }

  private async autoConnect() {
    if (process.env.NODE_ENV === 'development') {
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

  private setAccountHistory(accountHistory: AccountToken[]) {
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

      case 'blocked':
        actions.connection.blocked(tunnelState.details);
        break;
    }
  }

  private setSettings(newSettings: ISettings) {
    this.settings = newSettings;

    const reduxSettings = this.reduxActions.settings;
    const reduxAccount = this.reduxActions.account;

    reduxSettings.updateAllowLan(newSettings.allowLan);
    reduxSettings.updateEnableIpv6(newSettings.tunnelOptions.generic.enableIpv6);
    reduxSettings.updateBlockWhenDisconnected(newSettings.blockWhenDisconnected);
    reduxSettings.updateOpenVpnMssfix(newSettings.tunnelOptions.openvpn.mssfix);
    reduxSettings.updateBridgeState(newSettings.bridgeState);

    this.setRelaySettings(newSettings.relaySettings);

    if (newSettings.accountToken) {
      reduxAccount.updateAccountToken(newSettings.accountToken);
      reduxAccount.loggedIn();
    } else {
      reduxAccount.loggedOut();
    }
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

      case 'blocked':
        actions.updateBlockState(tunnelState.details.reason !== 'set_firewall_policy_error');
        break;
    }
  }

  private handleAccountChange(oldAccount?: string, newAccount?: string) {
    if (oldAccount && !newAccount) {
      if (this.loginTimer) {
        clearTimeout(this.loginTimer);
      }
      this.memoryHistory.replace('/login');
    } else if (!oldAccount && newAccount && !this.doingLogin) {
      this.memoryHistory.replace('/connect');
    }

    this.doingLogin = false;
  }

  private setLocation(location: ILocation) {
    this.reduxActions.connection.newLocation(location);
  }

  private setRelays(relayList: IRelayList) {
    const locations = relayList.countries
      .map((country) => ({
        name: country.name,
        code: country.code,
        hasActiveRelays: country.cities.some((city) => city.relays.length > 0),
        cities: country.cities
          .map((city) => ({
            name: city.name,
            code: city.code,
            latitude: city.latitude,
            longitude: city.longitude,
            hasActiveRelays: city.relays.length > 0,
            relays: city.relays,
          }))
          .sort((cityA, cityB) => cityA.name.localeCompare(cityB.name)),
      }))
      .sort((countryA, countryB) => countryA.name.localeCompare(countryB.name));

    this.reduxActions.settings.updateRelayLocations(locations);
  }

  private setCurrentVersion(versionInfo: ICurrentAppVersionInfo) {
    this.reduxActions.version.updateVersion(versionInfo.gui, versionInfo.isConsistent);
  }

  private setUpgradeVersion(upgradeVersion: IAppUpgradeInfo) {
    this.reduxActions.version.updateLatest(upgradeVersion);
  }

  private setGuiSettings(guiSettings: IGuiSettingsState) {
    this.guiSettings = guiSettings;
    this.reduxActions.settings.updateGuiSettings(guiSettings);
  }

  private setAccountExpiry(expiry?: string) {
    this.reduxActions.account.updateAccountExpiry(expiry);
  }

  private storeAutoStart(autoStart: boolean) {
    this.reduxActions.settings.updateAutoStart(autoStart);
  }

  private setWireguardPublicKey(publicKey?: string) {
    this.reduxActions.settings.setWireguardKey(publicKey);
  }
}
