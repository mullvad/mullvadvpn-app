// @flow

import React from 'react';
import { bindActionCreators } from 'redux';
import { Provider } from 'react-redux';
import {
  ConnectedRouter,
  push as pushHistory,
  replace as replaceHistory,
} from 'connected-react-router';
import { createMemoryHistory } from 'history';
import { webFrame, ipcRenderer } from 'electron';

import makeRoutes from './routes';
import { log } from './lib/platform';
import ReconnectionBackoff from './lib/reconnection-backoff';
import { DaemonRpc } from './lib/daemon-rpc';
import NotificationController from './notification-controller';
import { setShutdownHandler } from './shutdown-handler';
import { NoAccountError } from './errors';

import configureStore from './redux/store';
import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import daemonActions from './redux/daemon/actions';

import type { RpcCredentials } from './lib/rpc-address-file';
import type {
  DaemonRpcProtocol,
  AccountData,
  ConnectionObserver as DaemonConnectionObserver,
} from './lib/daemon-rpc';
import type { ReduxStore } from './redux/store';
import type { AccountToken, BackendState, RelaySettingsUpdate } from './lib/daemon-rpc';
import type { ConnectionState } from './redux/connection/reducers';
import type { TrayIconType } from './tray-icon-controller';

export default class AppRenderer {
  _notificationController = new NotificationController();
  _daemonRpc: DaemonRpcProtocol = new DaemonRpc();
  _reconnectBackoff = new ReconnectionBackoff();
  _credentials: ?RpcCredentials;
  _openConnectionObserver: ?DaemonConnectionObserver;
  _closeConnectionObserver: ?DaemonConnectionObserver;
  _memoryHistory = createMemoryHistory();
  _reduxStore: ReduxStore;
  _reduxActions: *;
  _accountDataState = new AccountDataState();

  constructor() {
    const store = configureStore(null, this._memoryHistory);
    const dispatch = store.dispatch;

    this._reduxStore = store;
    this._reduxActions = {
      account: bindActionCreators(accountActions, dispatch),
      connection: bindActionCreators(connectionActions, dispatch),
      settings: bindActionCreators(settingsActions, dispatch),
      daemon: bindActionCreators(daemonActions, dispatch),
      history: bindActionCreators(
        {
          push: pushHistory,
          replace: replaceHistory,
        },
        dispatch,
      ),
    };

    this._openConnectionObserver = this._daemonRpc.addOpenConnectionObserver(() => {
      this._onOpenConnection();
    });

    this._closeConnectionObserver = this._daemonRpc.addCloseConnectionObserver((error) => {
      this._onCloseConnection(error);
    });

    setShutdownHandler(async () => {
      log.info('Executing a shutdown handler');
      try {
        await this.disconnectTunnel();
        log.info('Disconnected the tunnel');
      } catch (e) {
        log.error(`Failed to shutdown tunnel: ${e.message}`);
      }
    });

    // disable pinch to zoom
    webFrame.setVisualZoomLevelLimits(1, 1);
  }

  dispose() {
    const openConnectionObserver = this._openConnectionObserver;
    const closeConnectionObserver = this._closeConnectionObserver;

    if (openConnectionObserver) {
      openConnectionObserver.unsubscribe();
      this._openConnectionObserver = null;
    }

    if (closeConnectionObserver) {
      closeConnectionObserver.unsubscribe();
      this._closeConnectionObserver = null;
    }
  }

  renderView() {
    return (
      <Provider store={this._reduxStore}>
        <ConnectedRouter history={this._memoryHistory}>
          {makeRoutes(this._reduxStore.getState, { app: this })}
        </ConnectedRouter>
      </Provider>
    );
  }

  connect() {
    this._connectToDaemon();
  }

  disconnect() {
    this._daemonRpc.disconnect();
  }

  async login(accountToken: AccountToken) {
    const actions = this._reduxActions;
    const history = this._memoryHistory;
    actions.account.startLogin(accountToken);

    log.debug('Logging in');

    try {
      const accountData = await this._daemonRpc.getAccountData(accountToken);
      await this._daemonRpc.setAccount(accountToken);

      actions.account.updateAccountExpiry(accountData.expiry);
      actions.account.loginSuccessful();

      // Redirect the user after some time to allow for
      // the 'Login Successful' screen to be visible
      setTimeout(async () => {
        if (history.location.pathname === '/login') {
          actions.history.replace('/connect');
        }

        try {
          log.debug('Auto-connecting the tunnel');
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

  async _restoreSession() {
    const actions = this._reduxActions;
    const history = this._memoryHistory;

    log.debug('Restoring session');

    const accountToken = await this._daemonRpc.getAccount();

    if (accountToken) {
      log.debug(`Got account token: ${accountToken}`);
      actions.account.updateAccountToken(accountToken);
      actions.account.loginSuccessful();

      // take user to main view if user is still at launch screen `/`
      if (history.location.pathname === '/') {
        actions.history.replace('/connect');
      }
    } else {
      log.debug('No account set, showing login view.');

      // show window when account is not set
      ipcRenderer.send('show-window');

      // take user to `/login` screen if user is at launch screen `/`
      if (history.location.pathname === '/') {
        actions.history.replace('/login');
      }
    }
  }

  async logout() {
    const actions = this._reduxActions;

    try {
      await Promise.all([
        this.disconnectTunnel(),
        this._daemonRpc.setAccount(null),
        this._fetchAccountHistory(),
      ]);
      actions.account.loggedOut();
      actions.history.replace('/login');

      // reset account data state on log out
      this._accountDataState = new AccountDataState();
    } catch (e) {
      log.info('Failed to logout: ', e.message);
    }
  }

  async connectTunnel() {
    const actions = this._reduxActions;

    try {
      const currentState = await this._daemonRpc.getState();
      if (currentState.state === 'secured') {
        log.debug('Refusing to connect as connection is already secured');
        actions.connection.connected();
      } else {
        actions.connection.connecting();
        await this._daemonRpc.connectTunnel();
      }
    } catch (error) {
      actions.connection.disconnected();
      throw error;
    }
  }

  disconnectTunnel() {
    return this._daemonRpc.disconnectTunnel();
  }

  updateRelaySettings(relaySettings: RelaySettingsUpdate) {
    return this._daemonRpc.updateRelaySettings(relaySettings);
  }

  async fetchRelaySettings() {
    const actions = this._reduxActions;
    const relaySettings = await this._daemonRpc.getRelaySettings();

    log.debug('Got relay settings from daemon', JSON.stringify(relaySettings));

    if (relaySettings.normal) {
      const payload = {};
      const normal = relaySettings.normal;
      const tunnel = normal.tunnel;
      const location = normal.location;

      if (location === 'any') {
        payload.location = 'any';
      } else {
        payload.location = location.only;
      }

      if (tunnel === 'any') {
        payload.port = 'any';
        payload.protocol = 'any';
      } else {
        const { port, protocol } = tunnel.only.openvpn;
        payload.port = port === 'any' ? port : port.only;
        payload.protocol = protocol === 'any' ? protocol : protocol.only;
      }

      actions.settings.updateRelay({
        normal: payload,
      });
    } else if (relaySettings.custom_tunnel_endpoint) {
      const custom_tunnel_endpoint = relaySettings.custom_tunnel_endpoint;
      const {
        host,
        tunnel: {
          openvpn: { port, protocol },
        },
      } = custom_tunnel_endpoint;

      actions.settings.updateRelay({
        custom_tunnel_endpoint: {
          host,
          port,
          protocol,
        },
      });
    }
  }

  async updateAccountExpiry() {
    const actions = this._reduxActions;
    const accountDataState = this._accountDataState;

    // Bail if something else requested an update to account data
    // or if account data cache was updated recently.
    if (accountDataState.isUpdating() || !accountDataState.needsUpdate()) {
      return;
    }

    try {
      const accountToken = await this._daemonRpc.getAccount();
      if (!accountToken) {
        throw new NoAccountError();
      }

      const accountData = await accountDataState.update(() => {
        return this._daemonRpc.getAccountData(accountToken);
      });

      // Check if account token is still the same after receiving account data
      const currentAccountToken = this._reduxStore.getState().account.accountToken;
      if (currentAccountToken === accountToken) {
        actions.account.updateAccountExpiry(accountData.expiry);
      }
    } catch (e) {
      log.error(`Failed to update account expiry: ${e.message}`);
    }
  }

  async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this._daemonRpc.removeAccountFromHistory(accountToken);
    await this._fetchAccountHistory();
  }

  async _fetchAccountHistory(): Promise<void> {
    const actions = this._reduxActions;

    const accountHistory = await this._daemonRpc.getAccountHistory();
    actions.account.updateAccountHistory(accountHistory);
  }

  async _fetchRelayLocations() {
    const actions = this._reduxActions;
    const locations = await this._daemonRpc.getRelayLocations();

    log.info('Got relay locations');

    const storedLocations = locations.countries.map((country) => ({
      name: country.name,
      code: country.code,
      hasActiveRelays: country.cities.some((city) => city.has_active_relays),
      cities: country.cities.map((city) => ({
        name: city.name,
        code: city.code,
        latitude: city.latitude,
        longitude: city.longitude,
        hasActiveRelays: city.has_active_relays,
      })),
    }));

    actions.settings.updateRelayLocations(storedLocations);
  }

  async _fetchLocation() {
    const actions = this._reduxActions;
    const location = await this._daemonRpc.getLocation();

    log.info('Got location from daemon');

    const locationUpdate = {
      ip: location.ip,
      country: location.country,
      city: location.city,
      latitude: location.latitude,
      longitude: location.longitude,
      mullvadExitIp: location.mullvad_exit_ip,
    };

    actions.connection.newLocation(locationUpdate);
  }

  async setAllowLan(allowLan: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setAllowLan(allowLan);
    actions.settings.updateAllowLan(allowLan);
  }

  async setEnableIpv6(enableIpv6: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setOpenVpnEnableIpv6(enableIpv6);
    actions.settings.updateEnableIpv6(enableIpv6);
  }

  async setAutoConnect(autoConnect: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setAutoConnect(autoConnect);
    actions.settings.updateAutoConnect(autoConnect);
  }

  async _fetchAllowLan() {
    const actions = this._reduxActions;
    const allowLan = await this._daemonRpc.getAllowLan();
    actions.settings.updateAllowLan(allowLan);
  }

  async _fetchAutoConnect() {
    const actions = this._reduxActions;
    const autoConnect = await this._daemonRpc.getAutoConnect();
    actions.settings.updateAutoConnect(autoConnect);
  }

  async _fetchSecurityState() {
    const securityState = await this._daemonRpc.getState();
    const connectionState = this._securityStateToConnectionState(securityState);
    this._updateConnectionState(connectionState);
  }

  async _fetchTunnelOptions() {
    const actions = this._reduxActions;
    const tunnelOptions = await this._daemonRpc.getTunnelOptions();
    actions.settings.updateEnableIpv6(tunnelOptions.openvpn.enableIpv6);
  }

  async _connectToDaemon(): Promise<void> {
    let credentials;
    try {
      credentials = await this._requestCredentials();
    } catch (error) {
      log.error(`Cannot request the RPC credentials: ${error.message}`);
      return;
    }

    this._credentials = credentials;
    this._daemonRpc.connect(credentials.connectionString);
  }

  async _onOpenConnection() {
    // save to redux that the app connected to daemon
    this._reduxActions.daemon.connected();

    // reset the reconnect backoff when connection established.
    this._reconnectBackoff.reset();

    // authenticate once connected
    const credentials = this._credentials;
    try {
      if (!credentials) {
        throw new Error('Credentials cannot be unset after connection is established.');
      }
      await this._authenticate(credentials.sharedSecret);
    } catch (error) {
      log.error(`Cannot authenticate: ${error.message}`);
    }

    // attempt to restore the session
    try {
      await this._restoreSession();
    } catch (error) {
      log.error(`Failed to restore session: ${error.message}`);
    }

    // make sure to re-subscribe to state notifications when connection is re-established.
    try {
      await this._subscribeStateListener();
    } catch (error) {
      log.error(`Cannot subscribe for RPC notifications: ${error.message}`);
    }

    // fetch initial state
    try {
      await this._fetchInitialState();
    } catch (error) {
      log.error(`Cannot fetch initial state: ${error.message}`);
    }

    // auto connect the tunnel on startup
    // note: disabled when developing
    if (process.env.NODE_ENV !== 'development') {
      try {
        log.debug('Auto-connecting the tunnel...');
        await this.connectTunnel();
      } catch (error) {
        log.error(`Failed to auto-connect the tunnel: ${error.message}`);
      }
    }
  }

  async _onCloseConnection(error: ?Error) {
    const actions = this._reduxActions;

    // save to redux that the app disconnected from daemon
    actions.daemon.disconnected();

    // recover connection on error
    if (error) {
      log.debug(`Lost connection to daemon: ${error.message}`);

      const recover = async () => {
        try {
          await this.connect();
        } catch (error) {
          log.error(`Failed to reconnect: ${error.message}`);
        }
      };

      this._reconnectBackoff.attempt(() => {
        recover();
      });

      // take user back to the launch screen `/`.
      actions.history.replace('/');
    }
  }

  _requestCredentials(): Promise<RpcCredentials> {
    return new Promise((resolve) => {
      ipcRenderer.once('daemon-connection-ready', (_event, credentials: RpcCredentials) => {
        resolve(credentials);
      });
      ipcRenderer.send('discover-daemon-connection');
    });
  }

  async _subscribeStateListener() {
    await this._daemonRpc.subscribeStateListener((newState, error) => {
      if (error) {
        log.error(`Received an error when processing the incoming state change: ${error.message}`);
      }

      if (newState) {
        const connectionState = this._securityStateToConnectionState(newState);

        log.debug(
          `Got new state from daemon {state: ${newState.state}, target_state: ${
            newState.target_state
          }}, translated to '${connectionState}'`,
        );

        this._updateConnectionState(connectionState);
        this._refreshStateOnChange();
      }
    });
  }

  _fetchInitialState() {
    return Promise.all([
      this._fetchSecurityState(),
      this.fetchRelaySettings(),
      this._fetchRelayLocations(),
      this._fetchAllowLan(),
      this._fetchAutoConnect(),
      this._fetchLocation(),
      this._fetchAccountHistory(),
      this._fetchTunnelOptions(),
    ]);
  }

  _updateTrayIcon(connectionState: ConnectionState) {
    const iconTypes: { [ConnectionState]: TrayIconType } = {
      connected: 'secured',
      connecting: 'securing',
    };
    const type = iconTypes[connectionState] || 'unsecured';

    ipcRenderer.send('change-tray-icon', type);
  }

  async _refreshStateOnChange() {
    try {
      await this._fetchLocation();
    } catch (error) {
      log.error(`Failed to fetch the location: ${error.message}`);
    }
  }

  _securityStateToConnectionState(backendState: BackendState): ConnectionState {
    if (backendState.state === 'unsecured' && backendState.target_state === 'secured') {
      return 'connecting';
    } else if (backendState.state === 'secured' && backendState.target_state === 'secured') {
      return 'connected';
    } else if (backendState.target_state === 'unsecured') {
      return 'disconnected';
    }
    throw new Error('Unsupported state/target state combination: ' + JSON.stringify(backendState));
  }

  _updateConnectionState(connectionState: ConnectionState) {
    const actions = this._reduxActions;
    switch (connectionState) {
      case 'connecting':
        actions.connection.connecting();
        break;
      case 'connected':
        actions.connection.connected();
        break;
      case 'disconnected':
        actions.connection.disconnected();
        break;
    }

    this._updateTrayIcon(connectionState);
    this._showNotification(connectionState);
  }

  _showNotification(connectionState: ConnectionState) {
    switch (connectionState) {
      case 'connecting':
        this._notificationController.show('Connecting');
        break;
      case 'connected':
        this._notificationController.show('Secured');
        break;
      case 'disconnected':
        this._notificationController.show('Unsecured');
        break;
      default:
        log.error(`Unexpected ConnectionState: ${(connectionState: empty)}`);
        return;
    }
  }

  async _authenticate(sharedSecret: string) {
    try {
      await this._daemonRpc.authenticate(sharedSecret);
      log.info('Authenticated with backend');
    } catch (e) {
      log.error(`Failed to authenticate with backend: ${e.message}`);
      throw e;
    }
  }
}

// Helper class to keep track of account data updates
class AccountDataState {
  _expiresAt: ?Date;
  _isUpdating = false;

  isUpdating() {
    return this._isUpdating;
  }

  needsUpdate() {
    return !this._expiresAt || this._expiresAt < new Date();
  }

  async update(fn: () => Promise<AccountData>): Promise<AccountData> {
    this._isUpdating = true;

    try {
      const accountData = await fn();
      this._expiresAt = new Date(Date.now() + 60 * 1000); // 60s expiration

      return accountData;
    } catch (error) {
      throw error;
    } finally {
      this._isUpdating = false;
    }
  }
}
