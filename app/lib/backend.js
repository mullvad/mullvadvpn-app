import moment from 'moment';
import Enum from './enum';
import { EventEmitter } from 'events';
import { servers } from '../config';
import Ipc from './ipc';

/**
 * Server info
 * @typedef {object} ServerInfo
 * @property {string}   address  - server address
 * @property {string}   name     - server name
 * @property {string}   city     - location city
 * @property {string}   country  - location country
 * @property {number[]} location - geo coordinate [latitude, longitude]
 *
 */

/**
 * Connect event
 *
 * @event Backend.EventType.connect
 * @param {string}     addr  - server address
 * @param {error|null} error - error
 */

/**
 * Connecting event
 *
 * @event Backend.EventType.connecting
 * @param {string} addr - server address
 */

/**
 * Disconnect event
 *
 * @event Backend.EventType.disconnect
 * @param {string} addr - server address
 */

/**
 * Login event
 *
 * @event Backend.EventType.login
 * @param {object} response
 * @param {error} error
 */

/**
 * Logging event
 *
 * @event Backend.EventType.logging
 * @param {object} response
 */

/**
 * Logout event
 *
 * @event Backend.EventType.logout
 */

/**
 * Updated IP event
 *
 * @event Backend.EventType.updatedIp
 * @param {string} new IP address
 */

/**
 * Updated location event
 *
 * @event Backend.EventType.updatedLocation
 * @param {object} location data
 */

/**
 * Updated reachability
 *
 * @event Backend.EventType.updatedReachability
 * @param {bool} true if online, otherwise false
 */

class BackendError extends Error {

  constructor(code) {
    super('');
    this.code = code;
    this.title = BackendError.localizedTitle(code);
    this.message = BackendError.localizedMessage(code);
  }

  static localizedTitle(code) {
    switch(code) {
    case Backend.ErrorType.noCredit:
      return 'Out of time';
    case Backend.ErrorType.noInternetConnection:
      return 'Offline';
    default:
      return 'Something went wrong';
    }
  }

  static localizedMessage(code) {
    switch(code) {
    case Backend.ErrorType.noCredit:
      return 'Buy more time, so you can continue using the internet securely';
    case Backend.ErrorType.noInternetConnection:
      return 'Your internet connection will be secured when you get back online';
    default:
      return '';
    }
  }

}

/**
 * Backend implementation
 * 
 * @class Backend
 */
export default class Backend extends EventEmitter {

  /**
   * BackendError type
   * 
   * @static
   * 
   * @memberOf Backend
   */
  static Error = BackendError;

  /**
   * Backend error enum
   * 
   * @static
   * 
   * @memberOf Backend
   */
  static ErrorType = new Enum({
    noCredit: 1,
    noInternetConnection: 2,
    invalidAccount: 3
  });

  /**
   * Event type enum
   * 
   * @type {EventType}
   * @extends {Enum}
   * @property {string} connect
   * @property {string} connecting
   * @property {string} disconnect
   * @property {string} login
   * @property {string} logging
   * @property {string} logout
   * @property {string} updatedIp
   * @property {string} updatedLocation
   * @property {string} updatedReachability
   */
  static EventType = new Enum('connect', 'connecting', 'disconnect', 'login', 'logging', 'logout', 'updatedIp', 'updatedLocation', 'updatedReachability');
  
  /**
   * Creates an instance of Backend.
   * 
   * @memberOf Backend
   */
  constructor(ipc) {
    super();
    this._ipc = ipc || new Ipc(undefined);
    this._registerIpcListeners();

    // check for network reachability
    this._startReachability();
  }

  setLocation(loc) {
    console.log('Got connection info to backend', loc);

    this._ipc = new Ipc(loc);
    this._registerIpcListeners();
  }

  /**
   * Tells whether account has credits
   * 
   * @type {bool}
   * @readonly
   * 
   * @memberOf Backend
   */
  get hasCredits() { 
    return this._paidUntil !== null && 
           moment(this._paidUntil).isAfter(moment()); 
  }

  sync() {
    console.log('Syncing with the backend...');

    this._ipc.send('getConnectionInfo')
      .then( connectionInfo => {
        console.log('Got connection info', connectionInfo);
        this.emit(Backend.EventType.updatedIp, connectionInfo.ip);
      })
      .catch(e => {
        console.log('Failed syncing with the backend', e);
      });

    this._ipc.send('getLocation')
      .then(location => {
        console.log('Got location', location);
        const newLocation = {
          location: location.latlong,
          country: location.country,
          city: location.city
        };
        this.emit(Backend.EventType.updatedLocation, newLocation, null);
      })
      .catch(e => {
        console.log('Failed getting new location', e);
      });
  }

  /**
   * Get server info by key
   * 'fastest' or 'nearest' can be used as well
   * 
   * @param {string} key 
   * @returns {ServerInfo}
   * 
   * @memberOf Backend
   */
  serverInfo(key) {
    switch(key) {
    case 'fastest': return this.fastestServer();
    case 'nearest': return this.nearestServer();
    default: return servers[key];
    }
  }

  /**
   * Get fastest server info
   * 
   * @returns {ServerInfo}
   * 
   * @memberOf Backend
   */
  fastestServer() {
    return {
      address: 'uk.mullvad.net',
      name: 'Fastest',
      city: 'London',
      country: 'United Kingdom',
      location: [51.5073509, -0.1277583]
    };
  }

  /**
   * Get nearest server info
   * 
   * @returns {ServerInfo}
   * 
   * @memberOf Backend
   */
  nearestServer() {
    return {
      address: 'es.mullvad.net',
      name: 'Nearest',
      city: 'Madrid',
      country: 'Spain',
      location: [40.4167754, -3.7037902]
    };
  }

  /**
   * Log in with mullvad account
   *
   * @emits Backend.EventType.logging
   * @emits Backend.EventType.login
   * @param {string} account 
   * 
   * @memberOf Backend
   */
  login(account) {
    console.log('Attempting to login with account number', account);
    this._paidUntil = null;

    // emit: logging in
    this.emit(Backend.EventType.logging, { account }, null);

    this._ipc.send('login', {
      accountNumber: account,
    }).then(response => {
      console.log('Successfully logged in', response);
      this._paidUntil = response.paidUntil;
      this.emit(Backend.EventType.login, response, undefined);
    }).catch(e => {
      console.warn('Failed to log in', e);
      const err = new BackendError(Backend.ErrorType.invalidAccount);
      this.emit(Backend.EventType.login, {}, err);
    });
  }

  /**
   * Log out
   *
   * @emits Backend.EventType.logout
   * @memberOf Backend
   */
  logout() {
    // @TODO: What does it mean for a logout to be successful or failed?
    this._ipc.send('logout')
      .then(() => {
        this._paidUntil = null;

        // emit event
        this.emit(Backend.EventType.logout);

        // disconnect user during logout
        this.disconnect();
      })
      .catch(e => {
        console.log('Failed to logout', e);
      });
  }

  /**
   * Connect to VPN server
   * @emits Backend.EventType.connecting
   * @emits Backend.EventType.connect
   *
   * @param {string} addr IP address or domain name
   * 
   * @memberOf Backend
   */
  connect(addr) {
    // do not attempt to connect when no credits available
    if(!this.hasCredits) {
      return;
    }

    // emit: connecting
    this.emit(Backend.EventType.connecting, addr);

    this._ipc.send('connect', { address: addr })
      .then(() => {
        this.emit(Backend.EventType.connect, addr);
        this.sync(); // TODO: This is a pooooooor way of updating the location and the IP and stuff
      })
      .catch(e => {
        console.log('Failed connecting to', addr, e);
        this.emit(Backend.EventType.connect, undefined, e);
      });
  }

  /**
   * Disconnect from VPN server
   * 
   * @emits Backend.EventType.disconnect
   * @memberOf Backend
   */
  disconnect() {
    // @TODO: Failure modes
    this._ipc.send('cancelConnection')
      .catch(e => {
        console.log('Failed cancelling connection', e);
      });

    // @TODO: Failure modes
    this._ipc.send('disconnect')
      .then(() => {
        // emit: disconnect
        this.emit(Backend.EventType.disconnect);
        this.sync(); // TODO: This is a pooooooor way of updating the location and the IP and stuff
      })
      .catch(e => {
        console.log('Failed to disconnect', e);
      });
  }

  /**
   * Start reachability monitoring for online/offline detection
   * This is currently done via HTML5 APIs but will be replaced later
   * with proper backend integration.
   * @private
   * @memberOf Backend
   * @emits Backend.EventType.updatedReachability
   */
  _startReachability() {
    // update online status in background
    setTimeout(() => {
      this.emit(Backend.EventType.updatedReachability, navigator.onLine);
    }, 0);

    window.addEventListener('online', () => {
      this.emit(Backend.EventType.updatedReachability, true);
    });

    window.addEventListener('offline', () => {
      // force disconnect since there is no real connection anyway.
      this.disconnect();
      this.emit(Backend.EventType.updatedReachability, false);
    });
  }


  _registerIpcListeners() {
    this._ipc.on('connection-info', (newConnectionInfo) => {
      console.log('Got new connection info from backend', newConnectionInfo);
    });
  }
}
