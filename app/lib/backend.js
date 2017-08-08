// @flow

import log from 'electron-log';
import EventEmitter from 'events';
import { servers } from '../config';
import { IpcFacade, RealIpc } from './ipc-facade';
import accountActions from '../redux/account/actions';
import connectionActions from '../redux/connection/actions';
import type { ReduxStore } from '../redux/store';
import { push } from 'react-router-redux';

import type { BackendState } from './ipc-facade';
import type { ConnectionState } from '../redux/connection/reducers';

export type EventType = 'connect' | 'connecting' | 'disconnect' | 'login' | 'logging' | 'logout' | 'updatedIp' | 'updatedLocation' | 'updatedReachability';
export type ErrorType = 'NO_CREDIT' | 'NO_INTERNET' | 'INVALID_ACCOUNT';

export type ServerInfo = {
  address: string,
  name: string,
  city: string,
  country: string,
  location: [number, number]
};

export type ServerInfoList = { [string]: ServerInfo };

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
    default:
      return '';
    }
  }

}

/**
 * Backend implementation
 */
export class Backend {

  _ipc: IpcFacade;
  _store: ReduxStore;
  _eventEmitter = new EventEmitter();

  constructor(store: ReduxStore, ipc: ?IpcFacade) {
    this._store = store;
    if(ipc) {
      this._ipc = ipc;
      this._registerIpcListeners();
      this._startReachability();
    }
  }

  setLocation(loc: string) {
    log.info('Got connection info to backend', loc);

    if (this._ipc) {
      this._ipc.setConnectionString(loc);
    } else {
      this._ipc = new RealIpc(loc);
    }
    this._registerIpcListeners();
  }

  sync() {
    log.info('Syncing with the backend...');

    this._ipc.getIp()
      .then( ip => {
        log.info('Got ip', ip);
        this._store.dispatch(connectionActions.newPublicIp(ip));
      })
      .catch(e => {
        log.info('Failed syncing with the backend', e);
      });

    this._ipc.getLocation()
      .then( location => {
        log.info('Got location', location);
        const newLocation = {
          location: location.latlong,
          country: location.country,
          city: location.city
        };
        this._store.dispatch(connectionActions.connectionChange(newLocation));
      })
      .catch(e => {
        log.info('Failed getting new location', e);
      });
  }

  serverInfo(identifier: string): ?ServerInfo {
    switch(identifier) {
    case 'fastest': return this.fastestServer();
    case 'nearest': return this.nearestServer();
    default: return (servers: ServerInfoList)[identifier];
    }
  }

  fastestServer(): ServerInfo {
    return {
      address: 'uk.mullvad.net',
      name: 'Fastest',
      city: 'London',
      country: 'United Kingdom',
      location: [51.5073509, -0.1277583]
    };
  }

  nearestServer(): ServerInfo {
    return {
      address: 'es.mullvad.net',
      name: 'Nearest',
      city: 'Madrid',
      country: 'Spain',
      location: [40.4167754, -3.7037902]
    };
  }


  login(accountNumber: string): Promise<void> {
    log.info('Attempting to login with account number', accountNumber);

    this._store.dispatch(accountActions.startLogin(accountNumber));

    return this._ipc.getAccountData(accountNumber)
      .then( response => {
        log.info('Account exists', response);

        return this._ipc.setAccount(accountNumber)
          .then( () => response );

      }).then( accountData => {
        log.info('Log in complete');

        this._store.dispatch(accountActions.loginSuccessful(accountData.paid_until));

        // Redirect the user after some time to allow for
        // the 'Login Successful' screen to be visible
        setTimeout(() => {
          this._store.dispatch(push('/connect'));
        }, 1000);
      }).catch(e => {
        log.error('Failed to log in', e);

        // TODO: This is not true. If there is a communication link failure the promise will be rejected too
        const err = new BackendError('INVALID_ACCOUNT');
        this._store.dispatch(accountActions.loginFailed(err));
      });
  }

  autologin() {
    log.info('Attempting to log in automatically');

    this._store.dispatch(accountActions.startLogin());

    return this._ipc.getAccount()
      .then( accountNumber => {
        if (!accountNumber) {
          throw new Error('No account set in the backend, failing autologin');
        }
        log.debug('The backend had an account number stored:', accountNumber);
        this._store.dispatch(accountActions.startLogin(accountNumber));

        return this._ipc.getAccountData(accountNumber);
      })
      .then( accountData => {
        log.info('The stored account number still exists', accountData);

        this._store.dispatch(accountActions.loginSuccessful(accountData.paid_until));

        this._store.dispatch(push('/connect'));
      })
      .catch( e => {
        log.warn('Unable to autologin', e);

        this._store.dispatch(accountActions.loginFailed(new BackendError('INVALID_ACCOUNT')));
        this._store.dispatch(push('/'));
      });
  }

  logout() {
    // @TODO: What does it mean for a logout to be successful or failed?
    this._ipc.setAccount('')
      .then(() => {

        this._store.dispatch(accountActions.loggedOut());

        // disconnect user during logout
        return this.disconnect()
          .then( () => {
            this._store.dispatch(push('/'));
          });
      })
      .catch(e => {
        log.info('Failed to logout', e);
      });
  }

  connect(addr: string) {

    this._store.dispatch(connectionActions.connectionChange({
      status: 'connecting',
      serverAddress: addr,
    }));


    this._ipc.setCountry(addr)
      .then( () => {
        return this._ipc.connect();
      })
      .catch(e => {
        log.info('Failed connecting to', addr, e);
        this._store.dispatch(connectionActions.connectionChange({
          status: 'disconnected',
        }));
      });
  }

  disconnect(): Promise<void> {
    // @TODO: Failure modes
    return this._ipc.disconnect()
      .catch(e => {
        log.info('Failed to disconnect', e);
      });
  }

  /**
   * Start reachability monitoring for online/offline detection
   * This is currently done via HTML5 APIs but will be replaced later
   * with proper backend integration.
   */
  _startReachability() {
    window.addEventListener('online', () => {
      this._store.dispatch(connectionActions.connectionChange({ isOnline: true }));
    });
    window.addEventListener('offline', () => {
      // force disconnect since there is no real connection anyway.
      this.disconnect();
      this._store.dispatch(connectionActions.connectionChange({ isOnline: false }));
    });

    // update online status in background
    setTimeout(() => {
      this._store.dispatch(connectionActions.connectionChange({ isOnline: navigator.onLine }));
    }, 0);
  }

  _registerIpcListeners() {
    this._ipc.registerStateListener(newState => {
      log.info('Got new state from backend', newState);

      const newStatus = this._securityStateToConnectionState(newState);
      this._store.dispatch(connectionActions.connectionChange({
        status: newStatus,
      }));

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
}
