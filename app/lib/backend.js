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

    this._ipc = new RealIpc(loc);
    this._registerIpcListeners();
  }

  sync() {
    log.info('Syncing with the backend...');

    this._ipc.getIp()
      .then( ip => {
        log.info('Got ip', ip);
        this._emit('updatedIp', ip);
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
        this._emit('updatedLocation', newLocation, null);
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


  login(accountNumber: string) {
    log.info('Attempting to login with account number', accountNumber);

    // emit: logging in
    this._emit('logging', { accountNumber: accountNumber }, null);

    this._store.dispatch(accountActions.loginChange({
      accountNumber: accountNumber,
      status: 'connecting',
      error: null,
    }));

    this._ipc.getAccountData(accountNumber)
      .then( response => {
        log.info('Account exists', response);

        return this._ipc.setAccount(accountNumber)
          .then( () => response );

      }).then( accountData => {
        log.info('Log in complete');

        this._emit('login', {
          paidUntil: accountData.paid_until,
        }, undefined);

        this._store.dispatch(accountActions.loginChange({
          status: 'ok',
          paidUntil: accountData.paid_until,
          error: null,
        }));

        // Redirect the user after some time to allow for
        // the 'Login Successful' screen to be visible
        setTimeout(() => {
          this._store.dispatch(push('/connect'));
        }, 1000);
      }).catch(e => {
        log.error('Failed to log in', e);

        // TODO: This is not true. If there is a communication link failure the promise will be rejected too
        const err = new BackendError('INVALID_ACCOUNT');
        this._store.dispatch(accountActions.loginChange({
          status: 'failed',
          error: err,
        }));

        this._emit('login', {}, err);
      });
  }


  logout() {
    // @TODO: What does it mean for a logout to be successful or failed?
    this._ipc.setAccount('')
      .then(() => {
        // emit event
        this._emit('logout');

        this._store.dispatch(accountActions.loginChange({
          status: 'none',
          accountNumber: null,
          paidUntil: null,
        }));

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

    // emit: connecting
    this._emit('connecting', addr);
    this._store.dispatch(connectionActions.connectionChange({
      status: 'connecting',
      serverAddress: addr,
    }));


    this._ipc.setCountry(addr)
      .then( () => {
        return this._ipc.connect();
      })
      .then(() => {
        this._emit('connect', addr);
        this._store.dispatch(connectionActions.connectionChange({
          status: 'connected',
          serverAddress: addr,
        }));

        this.sync(); // TODO: This is a pooooooor way of updating the location and the IP and stuff
      })
      .catch(e => {
        log.info('Failed connecting to', addr, e);
        this._emit('connect', undefined, e);
        this._store.dispatch(connectionActions.connectionChange({
          status: 'disconnected',
        }));
      });
  }

  disconnect(): Promise<void> {
    // @TODO: Failure modes
    return this._ipc.disconnect()
      .then(() => {
        // emit: disconnect
        this._emit('disconnect');
        this.sync(); // TODO: This is a pooooooor way of updating the location and the IP and stuff
      })
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
    window.addEventListener('online', () => this._emit('updatedReachability', true));
    window.addEventListener('offline', () => {
      // force disconnect since there is no real connection anyway.
      this.disconnect();
      this._emit('updatedReachability', false);
    });

    // update online status in background
    setTimeout(() => this._emit('updatedReachability', navigator.onLine), 0);
  }

  _registerIpcListeners() {
    this._ipc.registerStateListener(newState => {
      log.info('Got new state from backend', newState);

      const newStatus = this._backendStateToConnectionState(newState);
      this._store.dispatch(connectionActions.connectionChange({
        status: newStatus,
      }));
    });
  }

  _backendStateToConnectionState(backendState: BackendState): ConnectionState {
    switch(backendState) {
    case 'unsecured':
      return 'disconnected';
    case 'secured':
      return 'connected';

    default:
      throw new Error('Unknown backend state: ' + backendState);
    }
  }

  on(event: EventType, listener: Function) {
    this._eventEmitter.on(event, listener);
  }

  once(event: EventType, listener: Function) {
    this._eventEmitter.once(event, listener);
  }

  off(event: EventType, listener: Function) {
    this._eventEmitter.removeListener(event, listener);
  }

  _emit(event: EventType, ...args:Array<any>): boolean {
    return this._eventEmitter.emit(event, ...args);
  }
}
