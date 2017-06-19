// @flow
import log from 'electron-log';
import EventEmitter from 'events';
import { servers } from '../config';
import { IpcFacade, RealIpc } from './ipc-facade';

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
  _eventEmitter = new EventEmitter();

  constructor(ipc: ?IpcFacade) {
    this._ipc = ipc || new RealIpc('');
    this._registerIpcListeners();
    this._startReachability();
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

  login(account: string) {
    log.info('Attempting to login with account number', account);

    // emit: logging in
    this._emit('logging', { account }, null);

    this._ipc.getAccountData(account)
      .then( response => {
        log.info('Account exists', response);

        return this._ipc.setAccount(account)
          .then( () => response );

      }).then( accountData => {
        log.info('Log in complete');

        this._emit('login', {
          paidUntil: accountData.paid_until,
        }, undefined);

      }).catch(e => {
        log.error('Failed to log in', e);
        const err = new BackendError('INVALID_ACCOUNT');
        this._emit('login', {}, err);
      });
  }

  logout() {
    // @TODO: What does it mean for a logout to be successful or failed?
    this._ipc.setAccount('')
      .then(() => {
        // emit event
        this._emit('logout');

        // disconnect user during logout
        return this.disconnect();
      })
      .catch(e => {
        log.info('Failed to logout', e);
      });
  }

  connect(addr: string) {

    // emit: connecting
    this._emit('connecting', addr);

    this._ipc.setCountry(addr)
      .then( () => {
        return this._ipc.connect();
      })
      .then(() => {
        this._emit('connect', addr);
        this.sync(); // TODO: This is a pooooooor way of updating the location and the IP and stuff
      })
      .catch(e => {
        log.info('Failed connecting to', addr, e);
        this._emit('connect', undefined, e);
      });
  }

  disconnect() {
    // @TODO: Failure modes
    this._ipc.disconnect()
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
    /*this._ipc.on('connection-info', (newConnectionInfo) => {
      log.info('Got new connection info from backend', newConnectionInfo);
    });*/
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
