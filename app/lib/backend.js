// @flow

import { log } from '../lib/platform';
import EventEmitter from 'events';
import { IpcFacade, RealIpc } from './ipc-facade';
import accountActions from '../redux/account/actions';
import connectionActions from '../redux/connection/actions';
import settingsActions from '../redux/settings/actions';
import { push } from 'react-router-redux';

import type { ReduxStore } from '../redux/store';
import type { AccountToken, BackendState, RelaySettingsUpdate } from './ipc-facade';
import type { ConnectionState } from '../redux/connection/reducers';

export type ErrorType = 'NO_CREDIT' | 'NO_INTERNET' | 'INVALID_ACCOUNT' | 'NO_ACCOUNT';

export class BackendError extends Error {
  type: ErrorType;
  title: string;
  message: string;

  constructor(type: ErrorType) {
    super('');
    this.type = type;
    this.title = BackendError.localizedTitle(type);
    this.message = BackendError.localizedMessage(type);
  }

  static localizedTitle(type: ErrorType): string {
    switch(type) {
    case 'NO_CREDIT':
      return 'Out of time';
    case 'NO_INTERNET':
      return 'Offline';
    default:
      return 'Something went wrong';
    }
  }

  static localizedMessage(type: ErrorType): string {
    switch(type) {
    case 'NO_CREDIT':
      return 'Buy more time, so you can continue using the internet securely';
    case 'NO_INTERNET':
      return 'Your internet connection will be secured when you get back online';
    case 'INVALID_ACCOUNT':
      return 'Invalid account number';
    case 'NO_ACCOUNT':
      return 'No account was set';
    default:
      return '';
    }
  }

}


export type IpcCredentials = {
  connectionString: string,
  sharedSecret: string,
};
export function parseIpcCredentials(data: string): ?IpcCredentials {
  const [connectionString, sharedSecret] = data.split('\n', 2);
  if(connectionString && sharedSecret) {
    return {
      connectionString,
      sharedSecret,
    };
  } else {
    return null;
  }
}


/**
 * Backend implementation
 */
export class Backend {

  _ipc: IpcFacade;
  _credentials: ?IpcCredentials;
  _authenticationPromise: ?Promise<void>;
  _store: ReduxStore;
  _eventEmitter = new EventEmitter();

  constructor(store: ReduxStore, credentials?: IpcCredentials, ipc: ?IpcFacade) {
    this._store = store;
    this._credentials = credentials;


    if(ipc) {
      this._ipc = ipc;

      // force to re-authenticate when connection closed
      this._ipc.setCloseConnectionHandler(() => {
        this._authenticationPromise = null;
      });

      this._registerIpcListeners();
      this._startReachability();
    }
  }

  setCredentials(credentials: IpcCredentials) {
    log.debug('Got connection info to backend', credentials.connectionString);
    this._credentials = credentials;

    if (this._ipc) {
      this._credentials = credentials;
    } else {
      this._ipc = new RealIpc(credentials.connectionString);

      // force to re-authenticate when connection closed
      this._ipc.setCloseConnectionHandler(() => {
        this._authenticationPromise = null;
      });
    }
    this._registerIpcListeners();
  }

  async sync() {
    log.info('Syncing with the backend...');

    try {
      await this._fetchRelayLocations();
    } catch(e) {
      log.error('Failed to fetch the relay locations: ', e.message);
    }

    try {
      await this._fetchPublicIP();
    } catch(e) {
      log.error('Failed to fetch the public IP: ', e.message);
    }

    try {
      await this._fetchLocation();
    } catch(e) {
      log.error('Failed to fetch the location: ', e.message);
    }

    try {
      await this._fetchAllowLan();
    } catch(e) {
      log.error('Failed to fetch the LAN sharing policy: ', e.message);
    }

    await this._fetchAccountHistory();
  }

  async login(accountToken: AccountToken) {
    log.debug('Attempting to login');

    this._store.dispatch(accountActions.startLogin(accountToken));

    try {
      await this._ensureAuthenticated();

      const accountData = await this._ipc.getAccountData(accountToken);

      log.debug('Account exists', accountData);

      await this._ipc.setAccount(accountToken);

      log.info('Log in complete');

      this._store.dispatch(
        accountActions.loginSuccessful(accountData.expiry)
      );
      await this.fetchRelaySettings();

      // Redirect the user after some time to allow for
      // the 'Login Successful' screen to be visible
      setTimeout(() => {
        this._store.dispatch(push('/connect'));
        log.debug('Autoconnecting...');
        this.connect();
      }, 1000);

      await this._fetchAccountHistory();

    } catch(e) {
      log.error('Failed to log in,', e.message);

      // TODO: This is not true. If there is a communication link failure the promise will be rejected too
      const err = new BackendError('INVALID_ACCOUNT');
      this._store.dispatch(accountActions.loginFailed(err));
    }
  }

  async autologin() {
    try {
      log.debug('Attempting to log in automatically');

      await this._ensureAuthenticated();

      this._store.dispatch(accountActions.startLogin());

      const accountToken = await this._ipc.getAccount();
      if(!accountToken) {
        throw new BackendError('NO_ACCOUNT');
      }

      log.debug('The backend had an account number stored: ', accountToken);
      this._store.dispatch(accountActions.startLogin(accountToken));

      const accountData = await this._ipc.getAccountData(accountToken);
      log.debug('The stored account number still exists', accountData);

      this._store.dispatch(accountActions.loginSuccessful(accountData.expiry));
      this._store.dispatch(push('/connect'));
    } catch (e) {
      log.warn('Unable to autologin,', e.message);

      this._store.dispatch(accountActions.autoLoginFailed());
      this._store.dispatch(push('/'));

      throw e;
    }
  }

  async logout() {
    // @TODO: What does it mean for a logout to be successful or failed?
    try {
      await this._ensureAuthenticated();
      await this._ipc.setAccount(null);

      this._store.dispatch(accountActions.loggedOut());

      // disconnect user during logout
      await this.disconnect();

      this._store.dispatch(push('/'));
    } catch (e) {
      log.info('Failed to logout: ', e.message);
    }
  }

  async connect() {
    try {
      this._store.dispatch(connectionActions.connecting());

      await this._ensureAuthenticated();
      await this._ipc.connect();
    } catch (e) {
      log.error('Failed to connect: ', e.message);
      this._store.dispatch(connectionActions.disconnected());
    }
  }

  async disconnect() {
    // @TODO: Failure modes
    try {
      await this._ensureAuthenticated();
      await this._ipc.disconnect();
    } catch (e) {
      log.error('Failed to disconnect: ', e.message);
    }
  }

  async shutdown() {
    try {
      await this._ensureAuthenticated();
      await this._ipc.shutdown();
    } catch (e) {
      log.error('Failed to shutdown: ', e.message);
    }
  }

  async updateRelaySettings(relaySettings: RelaySettingsUpdate) {
    try {
      await this._ensureAuthenticated();
      await this._ipc.updateRelaySettings(relaySettings);
    } catch (e) {
      log.error('Failed to update relay settings: ', e.message);
    }
  }

  async fetchRelaySettings() {
    await this._ensureAuthenticated();

    const relaySettings = await this._ipc.getRelaySettings();
    log.debug('Got relay settings from backend', JSON.stringify(relaySettings));

    if(relaySettings.normal) {
      const payload = {};
      const normal = relaySettings.normal;
      const tunnel = normal.tunnel;
      const location = normal.location;

      if(location === 'any') {
        payload.location = 'any';
      } else {
        payload.location = location.only;
      }

      if(tunnel === 'any') {
        payload.port = 'any';
        payload.protocol = 'any';
      } else {
        const { port, protocol } = tunnel.only.openvpn;
        payload.port = port === 'any' ? port : port.only;
        payload.protocol = protocol === 'any' ? protocol : protocol.only;
      }

      this._store.dispatch(
        settingsActions.updateRelay({
          normal: payload
        })
      );
    } else if(relaySettings.custom_tunnel_endpoint) {
      const custom_tunnel_endpoint = relaySettings.custom_tunnel_endpoint;
      const { host, tunnel: { openvpn: { port, protocol } } } = custom_tunnel_endpoint;

      this._store.dispatch(
        settingsActions.updateRelay({
          custom_tunnel_endpoint: {
            host, port, protocol
          }
        })
      );
    }
  }

  async removeAccountFromHistory(accountToken: AccountToken) {
    try {
      await this._ensureAuthenticated();
      await this._ipc.removeAccountFromHistory(accountToken);
      await this._fetchAccountHistory();
    } catch(e) {
      log.error('Failed to remove account token from history', e.message);
    }
  }

  async _fetchAccountHistory() {
    try {
      await this._ensureAuthenticated();
      const accountHistory = await this._ipc.getAccountHistory();
      this._store.dispatch(
        accountActions.updateAccountHistory(accountHistory)
      );
    } catch(e) {
      log.info('Failed to fetch account history,', e.message);
      throw e;
    }
  }

  async _fetchRelayLocations() {
    await this._ensureAuthenticated();

    const locations = await this._ipc.getRelayLocations();

    log.info('Got relay locations');

    const storedLocations = locations.countries.map((country) => ({
      name: country.name,
      code: country.code,
      hasActiveRelays: country.cities.some((city) => city.has_active_relays),
      cities: country.cities.map((city) => ({
        name: city.name,
        code: city.code,
        position: city.position,
        hasActiveRelays: city.has_active_relays,
      }))
    }));

    this._store.dispatch(
      settingsActions.updateRelayLocations(storedLocations)
    );
  }

  async _fetchPublicIP() {
    await this._ensureAuthenticated();

    const publicIp = await this._ipc.getPublicIp();

    log.info('Got public IP: ', publicIp);

    this._store.dispatch(
      connectionActions.newPublicIp(publicIp)
    );
  }

  async _fetchLocation() {
    await this._ensureAuthenticated();

    const location = await this._ipc.getLocation();

    log.info('Got location: ', location);

    const locationUpdate = {
      country: location.country,
      city: location.city,
      location: location.position
    };

    this._store.dispatch(
      connectionActions.newLocation(locationUpdate)
    );
  }

  async setAllowLan(allowLan: boolean) {
    try {
      await this._ensureAuthenticated();
      await this._ipc.setAllowLan(allowLan);

      this._store.dispatch(
        settingsActions.updateAllowLan(allowLan)
      );
    } catch(e) {
      log.error('Failed to change the LAN sharing policy: ', e.message);
    }
  }

  async _fetchAllowLan() {
    await this._ensureAuthenticated();

    const allowLan = await this._ipc.getAllowLan();
    this._store.dispatch(
      settingsActions.updateAllowLan(allowLan)
    );
  }

  async fetchSecurityState() {
    await this._ensureAuthenticated();

    const securityState = await this._ipc.getState();
    const connectionState = this._securityStateToConnectionState(securityState);
    this._dispatchConnectionState(connectionState);
  }

  /**
   * Start reachability monitoring for online/offline detection
   * This is currently done via HTML5 APIs but will be replaced later
   * with proper backend integration.
   */
  _startReachability() {
    window.addEventListener('online', () => {
      this._store.dispatch(connectionActions.online());
    });
    window.addEventListener('offline', () => {
      // force disconnect since there is no real connection anyway.
      this.disconnect();
      this._store.dispatch(connectionActions.offline());
    });

    // update online status in background
    setTimeout(() => {
      const action = navigator.onLine
        ? connectionActions.online()
        : connectionActions.offline();

      this._store.dispatch(action);
    }, 0);
  }

  async _registerIpcListeners() {
    await this._ensureAuthenticated();
    this._ipc.registerStateListener(newState => {
      log.debug('Got new state from backend', newState);

      const newStatus = this._securityStateToConnectionState(newState);
      this._dispatchConnectionState(newStatus);
      this.sync();
    });
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

  _dispatchConnectionState(connectionState: ConnectionState) {
    switch(connectionState) {
    case 'connecting':
      this._store.dispatch(connectionActions.connecting());
      break;
    case 'connected':
      this._store.dispatch(connectionActions.connected());
      break;
    case 'disconnected':
      this._store.dispatch(connectionActions.disconnected());
      break;
    }
  }

  _ensureAuthenticated() {
    const credentials = this._credentials;
    if(credentials) {
      if(!this._authenticationPromise) {
        this._authenticationPromise = this._authenticate(credentials.sharedSecret);
      }
      return this._authenticationPromise;
    } else {
      return Promise.reject(new Error('Missing authentication credentials.'));
    }
  }

  async _authenticate(sharedSecret: string) {
    try {
      await this._ipc.authenticate(sharedSecret);
      log.info('Authenticated with backend');
    } catch (e) {
      log.error('Failed to authenticate with backend: ', e.message);
      throw e;
    }
  }
}
