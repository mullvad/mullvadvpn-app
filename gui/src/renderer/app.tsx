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

import { InvalidAccountError } from '../main/errors';
import AppRoutes from './routes';

import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import configureStore from './redux/store';
import userInterfaceActions from './redux/userinterface/actions';
import versionActions from './redux/version/actions';

import { IAppUpgradeInfo, ICurrentAppVersionInfo } from '../main';
import { IWindowShapeParameters } from '../main/window-controller';
import { cities, countries, loadTranslations, messages, relayLocations } from '../shared/gettext';
import { IGuiSettingsState } from '../shared/gui-settings-state';
import { IpcRendererEventChannel } from '../shared/ipc-event-channel';
import AccountDataCache, { AccountFetchRetryAction } from './lib/account-data-cache';
import AccountExpiry from './lib/account-expiry';

import {
  AccountToken,
  ILocation,
  IRelayList,
  ISettings,
  RelaySettings,
  RelaySettingsUpdate,
  TunnelStateTransition,
} from '../shared/daemon-rpc-types';

type AccountVerification = { status: 'verified' } | { status: 'deferred'; error: Error };

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
  private accountDataCache = new AccountDataCache(
    (accountToken) => {
      return IpcRendererEventChannel.account.getData(accountToken);
    },
    (accountData) => {
      this.setAccountExpiry(accountData && accountData.expiry);
    },
  );

  private locale: string;
  private tunnelState: TunnelStateTransition;
  private settings: ISettings;
  private guiSettings: IGuiSettingsState;
  private accountExpiry?: AccountExpiry;
  private connectedToDaemon = false;
  private autoConnected = false;
  private doingLogin = false;
  private loginTimer?: NodeJS.Timeout;

  constructor() {
    ipcRenderer.on(
      'update-window-shape',
      (_event: Electron.Event, shapeParams: IWindowShapeParameters) => {
        if (typeof shapeParams.arrowPosition === 'number') {
          this.reduxActions.userInterface.updateWindowArrowPosition(shapeParams.arrowPosition);
        }
      },
    );

    ipcRenderer.on('window-shown', () => {
      if (this.connectedToDaemon) {
        this.updateAccountExpiry();
      }
    });

    IpcRendererEventChannel.daemonConnected.listen(() => {
      this.onDaemonConnected();
    });

    IpcRendererEventChannel.daemonDisconnected.listen((errorMessage?: string) => {
      this.onDaemonDisconnected(errorMessage ? new Error(errorMessage) : undefined);
    });

    IpcRendererEventChannel.accountHistory.listen((newAccountHistory: AccountToken[]) => {
      this.setAccountHistory(newAccountHistory);
    });

    IpcRendererEventChannel.tunnel.listen((newState: TunnelStateTransition) => {
      this.setTunnelState(newState);
      this.updateBlockedState(newState, this.settings.blockWhenDisconnected);

      if (this.accountExpiry) {
        this.detectStaleAccountExpiry(newState, this.accountExpiry);
      }
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

    // Request the initial state from the main process
    const initialState = IpcRendererEventChannel.state.get();

    this.locale = initialState.locale;
    this.tunnelState = initialState.tunnelState;
    this.settings = initialState.settings;
    this.guiSettings = initialState.guiSettings;

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
          <AppRoutes
            sharedProps={{
              app: this,
              locale: this.locale,
            }}
          />
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
      const verification = await this.verifyAccount(accountToken);

      if (verification.status === 'deferred') {
        log.warn(`Failed to get account data, logging in anyway: ${verification.error.message}`);
      }

      await IpcRendererEventChannel.account.set(accountToken);

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
      log.error('Failed to log in,', error.message);

      actions.account.loginFailed(error);
    }
  }

  public verifyAccount(accountToken: AccountToken): Promise<AccountVerification> {
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

  public async logout() {
    try {
      await IpcRendererEventChannel.account.unset();
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

  public updateAccountExpiry() {
    if (this.settings.accountToken) {
      this.accountDataCache.fetch(this.settings.accountToken);
    }
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

  public async setAutoConnect(autoConnect: boolean) {
    return IpcRendererEventChannel.guiSettings.setAutoConnect(autoConnect);
  }

  public async setAutoStart(autoStart: boolean): Promise<void> {
    this.storeAutoStart(autoStart);

    return IpcRendererEventChannel.autoStart.set(autoStart);
  }

  public setStartMinimized(startMinimized: boolean) {
    IpcRendererEventChannel.guiSettings.setStartMinimized(startMinimized);
  }

  public setMonochromaticIcon(monochromaticIcon: boolean) {
    IpcRendererEventChannel.guiSettings.setMonochromaticIcon(monochromaticIcon);
  }

  private setRelaySettings(relaySettings: RelaySettings) {
    const actions = this.reduxActions;

    if ('normal' in relaySettings) {
      const normal = relaySettings.normal;
      const tunnel = normal.tunnel;
      const location = normal.location;

      const relayLocation = location === 'any' ? 'any' : location.only;

      if (tunnel === 'any') {
        actions.settings.updateRelay({
          normal: {
            location: relayLocation,
            port: 'any',
            protocol: 'any',
          },
        });
      } else {
        const constraints = tunnel.only;

        if ('openvpn' in constraints) {
          const { port, protocol } = constraints.openvpn;

          actions.settings.updateRelay({
            normal: {
              location: relayLocation,
              port: port === 'any' ? port : port.only,
              protocol: protocol === 'any' ? protocol : protocol.only,
            },
          });
        } else if ('wireguard' in constraints) {
          const { port } = constraints.wireguard;

          actions.settings.updateRelay({
            normal: {
              location: relayLocation,
              port: port === 'any' ? port : port.only,
              protocol: 'udp',
            },
          });
        }
      }
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

  private setTunnelState(tunnelState: TunnelStateTransition) {
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

    this.setRelaySettings(newSettings.relaySettings);

    if (newSettings.accountToken) {
      reduxAccount.updateAccountToken(newSettings.accountToken);
      reduxAccount.loggedIn();
    } else {
      reduxAccount.loggedOut();
    }
  }

  private updateBlockedState(tunnelState: TunnelStateTransition, blockWhenDisconnected: boolean) {
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
      this.accountDataCache.invalidate();

      if (this.loginTimer) {
        clearTimeout(this.loginTimer);
      }

      this.memoryHistory.replace('/login');
    } else if (!oldAccount && newAccount) {
      this.accountDataCache.fetch(newAccount);

      if (!this.doingLogin) {
        this.memoryHistory.replace('/connect');
      }
    } else if (oldAccount && newAccount && oldAccount !== newAccount) {
      this.accountDataCache.fetch(newAccount);
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
    this.accountExpiry = expiry ? new AccountExpiry(expiry, this.locale) : undefined;
    this.reduxActions.account.updateAccountExpiry(expiry);
  }

  private detectStaleAccountExpiry(
    tunnelState: TunnelStateTransition,
    accountExpiry: AccountExpiry,
  ) {
    // It's likely that the account expiry is stale if the daemon managed to establish the tunnel.
    if (tunnelState.state === 'connected' && accountExpiry.hasExpired()) {
      log.info('Detected the stale account expiry.');
      this.accountDataCache.invalidate();
    }
  }

  private storeAutoStart(autoStart: boolean) {
    this.reduxActions.settings.updateAutoStart(autoStart);
  }
}
