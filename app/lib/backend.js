// @flow

import { ipcRenderer } from 'electron';
import { bindActionCreators } from 'redux';
import { push as pushHistory } from 'react-router-redux';
import {
  RemoteError as JsonRpcTransportRemoteError,
  TimeOutError as JsonRpcTransportTimeOutError,
} from './jsonrpc-transport';
import accountActions from '../redux/account/actions';
import connectionActions from '../redux/connection/actions';
import settingsActions from '../redux/settings/actions';
import { log } from '../lib/platform';

import type { RpcCredentials as OriginalRpcCredentials } from './rpc-address-file';
import type {
  DaemonRpcProtocol,
  ConnectionObserver as DaemonConnectionObserver,
} from './daemon-rpc';
import type { ReduxStore } from '../redux/store';
import type { AccountToken, BackendState, RelaySettingsUpdate } from './daemon-rpc';
import type { ConnectionState } from '../redux/connection/reducers';

export class NoCreditError extends Error {
  constructor() {
    super("Account doesn't have enough credit available for connection");
  }

  get userFriendlyTitle(): string {
    return 'Out of time';
  }

  get userFriendlyMessage(): string {
    return 'Buy more time, so you can continue using the internet securely';
  }
}

export class NoInternetError extends Error {
  constructor() {
    super('Internet connectivity is currently unavailable');
  }

  get userFriendlyTitle(): string {
    return 'Offline';
  }

  get userFriendlyMessage(): string {
    return 'Your internet connection will be secured when you get back online';
  }
}

export class NoDaemonError extends Error {
  constructor() {
    super('Could not connect to Mullvad daemon');
  }
}

export class InvalidAccountError extends Error {
  constructor() {
    super('Invalid account number');
  }
}

export class NoAccountError extends Error {
  constructor() {
    super('No account was set');
  }
}

export class CommunicationError extends Error {
  constructor() {
    super('api.mullvad.net is blocked, please check your firewall');
  }
}

export class UnknownError extends Error {
  constructor(cause: string) {
    super(`An unknown error occurred, ${cause}`);
  }

  get userFriendlyTitle(): string {
    return 'Something went wrong';
  }
}

export class CredentialsRequestError extends Error {
  _reason: Error;

  constructor(reason: Error) {
    super('Failed to request the RPC credentials');
    this._reason = reason;
  }

  get reason(): Error {
    return this._reason;
  }
}

export type RpcCredentials = OriginalRpcCredentials;
export interface RpcCredentialsProvider {
  request(): Promise<RpcCredentials>;
}

/**
 * Backend implementation
 */
export class Backend {
  _daemonRpc: DaemonRpcProtocol;
  _credentialsProvider: RpcCredentialsProvider;
  _reconnectBackoff = new ReconnectionBackoff();
  _credentials: ?RpcCredentials;
  _openConnectionObserver: ?DaemonConnectionObserver;
  _closeConnectionObserver: ?DaemonConnectionObserver;
  _reduxActions: *;

  constructor(
    store: ReduxStore,
    rpc: DaemonRpcProtocol,
    credentialsProvider: RpcCredentialsProvider,
  ) {
    this._daemonRpc = rpc;
    this._credentialsProvider = credentialsProvider;

    this._reduxActions = {
      account: bindActionCreators(accountActions, store.dispatch),
      connection: bindActionCreators(connectionActions, store.dispatch),
      settings: bindActionCreators(settingsActions, store.dispatch),
      history: bindActionCreators({ push: pushHistory }, store.dispatch),
    };

    this._openConnectionObserver = rpc.addOpenConnectionObserver(() => {
      this._onOpenConnection();
    });

    this._closeConnectionObserver = rpc.addCloseConnectionObserver((error) => {
      this._onCloseConnection(error);
    });

    this._setupReachability();
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

  connect() {
    this._connectToDaemon();
  }

  disconnect() {
    this._daemonRpc.disconnect();
  }

  async login(accountToken: AccountToken) {
    const actions = this._reduxActions;
    actions.account.startLogin(accountToken);

    log.debug('Attempting to login');

    try {
      const accountData = await this._daemonRpc.getAccountData(accountToken);
      await this._daemonRpc.setAccount(accountToken);

      actions.account.loginSuccessful(accountData.expiry);

      // Redirect the user after some time to allow for
      // the 'Login Successful' screen to be visible
      setTimeout(() => {
        actions.history.push('/connect');
        log.debug('Autoconnecting...');
        this.connectTunnel();
      }, 1000);
    } catch (error) {
      log.error('Failed to log in,', error.message);

      actions.account.loginFailed(this._rpcErrorToBackendError(error));
    }
  }

  async autologin() {
    const actions = this._reduxActions;
    actions.account.startLogin();

    log.debug('Attempting to log in automatically');

    try {
      const accountToken = await this._daemonRpc.getAccount();
      if (!accountToken) {
        throw new NoAccountError();
      }

      log.debug(`The backend had an account number stored: ${accountToken}`);
      actions.account.startLogin(accountToken);

      const accountData = await this._daemonRpc.getAccountData(accountToken);
      log.debug('The stored account number still exists:', accountData);

      actions.account.loginSuccessful(accountData.expiry);
      actions.history.push('/connect');
    } catch (e) {
      log.warn('Unable to autologin,', e.message);

      actions.account.autoLoginFailed();
      actions.history.push('/');

      throw e;
    }
  }

  async logout() {
    const actions = this._reduxActions;

    try {
      await Promise.all([this.disconnectTunnel(), this._daemonRpc.setAccount(null)]);
      actions.account.loggedOut();
      actions.history.push('/');
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

    log.debug('Got relay settings from backend', JSON.stringify(relaySettings));

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
    try {
      const accountToken = await this._daemonRpc.getAccount();
      if (!accountToken) {
        throw new NoAccountError();
      }
      const accountData = await this._daemonRpc.getAccountData(accountToken);
      actions.account.updateAccountExpiry(accountData.expiry);
    } catch (e) {
      log.error(`Failed to update account expiry: ${e.message}`);
    }
  }

  async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this._daemonRpc.removeAccountFromHistory(accountToken);
    await this.fetchAccountHistory();
  }

  async fetchAccountHistory(): Promise<void> {
    const actions = this._reduxActions;

    const accountHistory = await this._daemonRpc.getAccountHistory();
    actions.account.updateAccountHistory(accountHistory);
  }

  async fetchRelayLocations() {
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

  async fetchLocation() {
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

  async fetchAllowLan() {
    const actions = this._reduxActions;
    const allowLan = await this._daemonRpc.getAllowLan();
    actions.settings.updateAllowLan(allowLan);
  }

  async fetchSecurityState() {
    const securityState = await this._daemonRpc.getState();
    const connectionState = this._securityStateToConnectionState(securityState);
    this._updateConnectionState(connectionState);
  }

  async _requestCredentials(): Promise<RpcCredentials> {
    try {
      return await this._credentialsProvider.request();
    } catch (providerError) {
      throw new CredentialsRequestError(providerError);
    }
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

    // autologin
    try {
      await this.autologin();
    } catch (error) {
      if (error instanceof NoAccountError) {
        log.debug('No previously configured account set, showing window');
        ipcRenderer.send('show-window');
      } else {
        log.error(`Failed to autologin: ${error.message}`);
      }
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

    // auto connect the tunnel
    try {
      await this.connectTunnel();
    } catch (error) {
      log.error(`Cannot autoconnect the tunnel: ${error.message}`);
    }
  }

  async _onCloseConnection(error: ?Error) {
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
    }
  }

  /**
   * Start reachability monitoring for online/offline detection
   * This is currently done via HTML5 APIs but will be replaced later
   * with proper backend integration.
   */
  _setupReachability() {
    const actions = this._reduxActions;

    window.addEventListener('online', () => {
      actions.connection.online();
    });
    window.addEventListener('offline', () => {
      actions.connection.offline();
    });

    if (navigator.onLine) {
      actions.connection.online();
    } else {
      actions.connection.offline();
    }
  }

  async _subscribeStateListener() {
    await this._daemonRpc.subscribeStateListener((newState, error) => {
      if (error) {
        log.error(`Received an error when processing the incoming state change: ${error.message}`);
      }

      if (newState) {
        const connectionState = this._securityStateToConnectionState(newState);

        log.debug(
          `Got new state from backend {state: ${newState.state}, target_state: ${
            newState.target_state
          }}, translated to '${connectionState}'`,
        );

        this._updateConnectionState(connectionState);
        this._refreshStateOnChange();
      }
    });
  }

  async _fetchInitialState() {
    return Promise.all([
      this.fetchSecurityState(),
      this.fetchRelaySettings(),
      this.fetchRelayLocations(),
      this.fetchAllowLan(),
      this.fetchLocation(),
    ]);
  }

  async _refreshStateOnChange() {
    try {
      await this.fetchLocation();
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
  }

  _rpcErrorToBackendError(e) {
    if (e instanceof JsonRpcTransportRemoteError) {
      switch (e.code) {
        case -200: // Account doesn't exist
          return new InvalidAccountError();
        case -32603: // Internal error
          // We treat all internal backend errors as the user cannot reach
          // api.mullvad.net. This is not always true of course, but it is
          // true so often that we choose to disregard the other edge cases
          // for now.
          return new CommunicationError();
      }
    } else if (e instanceof JsonRpcTransportTimeOutError) {
      return new CommunicationError();
    } else if (e instanceof NoDaemonError) {
      return e;
    }

    return new UnknownError(e.message);
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

/*
 * Used to calculate the time to wait before reconnecting
 * the websocket.
 *
 * It uses a linear backoff function that goes from 500ms
 * to 3000ms
 */
class ReconnectionBackoff {
  _attempt: number;

  constructor() {
    this._attempt = 0;
  }

  attempt(handler: () => void) {
    setTimeout(handler, this._getIncreasedBackoff());
  }

  reset() {
    this._attempt = 0;
  }

  _getIncreasedBackoff() {
    if (this._attempt < 6) {
      this._attempt++;
    }
    return this._attempt * 500;
  }
}
