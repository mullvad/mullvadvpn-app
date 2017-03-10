import moment from 'moment';
import Enum from './enum';
import { EventEmitter } from 'events';
import { servers } from '../config';
import { ConnectionState as ReduxConnectionState } from '../enums';

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
 * Backend implementation
 * 
 * @class Backend
 */
export default class Backend extends EventEmitter {

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
   */
  static EventType = new Enum('connect', 'connecting', 'disconnect', 'login', 'logging', 'logout', 'updatedIp', 'updatedLocation');

  /**
   * Connection state enum
   * 
   * @type {ConnectionState}
   * @extends {Enum}
   * @property {string} disconnected  - Initial state (disconnected)
   * @property {string} connecting    - Connecting
   * @property {string} connected     - Connected
   */
  static ConnectionState = new Enum('disconnected', 'connecting', 'connected');
  
  /**
   * Creates an instance of Backend.
   * 
   * @memberOf Backend
   */
  constructor() {
    super();
    this._account = null;
    this._paidUntil = null;
    this._serverAddress = null;
    this._connStatus = Backend.ConnectionState.disconnected;
    this._cancellationHandler = null;

    // update IP in background
    setTimeout(::this._refreshIp, 0);
  }

  // Accessors

  /**
   * Account number
   * 
   * @type {string}
   * @readonly
   * 
   * @memberOf Backend
   */
  get account() { return this._account; }

  /**
   * Until when services are paid for (ISO string)
   * 
   * @type {string}
   * @readonly
   * 
   * @memberOf Backend
   */
  get paidUntil() { return this._paidUntil; }

  /**
   * Server IP address or domain name
   * 
   * @type {string}
   * @readonly
   * 
   * @memberOf Backend
   */
  get serverAddress() { return this._serverAddress; }

  // Public methods

  /**
   * Patch backend state.
   * 
   * Currently backend does not have external state
   * such as VPN connection status or IP address.
   * 
   * So far we store everything in redux and have to
   * sync redux state with backend.
   * 
   * In future this will be the other way around.
   * 
   * @param {Redux.Store} store - an instance of Redux store
   * 
   * @memberOf Backend
   */
  syncWithReduxStore(store) {
    const mapConnStatus = (s) => {
      const S = ReduxConnectionState;
      const BS = Backend.ConnectionState;
      switch(s) {
      case S.connected: return BS.connected;
      case S.connecting: return BS.connecting;
      default: return BS.disconnected;
      }
    };
    const { user, connect } = store.getState();
    const server = this.serverInfo(connect.preferredServer);

    if(server) {
      this._serverAddress = server.address;
    }
    
    if(user.account) {
      this._account = user.account;
    }

    if(user.paidUntil) {
      this._paidUntil = user.paidUntil;
    }

    this._connStatus = mapConnStatus(connect.status);
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
   * Get fastest server
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
    this._account = account;
    this._paidUntil = null;

    // emit: logging in
    this.emit(Backend.EventType.logging, { account, paidUntil: this._paidUntil }, null);

    // @TODO: Add login call
    setTimeout(() => {
      let err = null;
      let res = { account };
      
      if(account.startsWith('1111')) { // accounts starting with 1111 expire in one month
        res.paidUntil = moment().startOf('day').add(15, 'days').toISOString();
      } else if(account.startsWith('2222')) { // expired in 2013
        res.paidUntil = moment('2013-01-01').toISOString();
      } else if(account.startsWith('3333')) { // expire in 2038
        res.paidUntil = moment('2038-01-01').toISOString();
      } else {
        err = new Error('Invalid account number.');
      }

      // emit: login
      this.emit(Backend.EventType.login, res, err);
    }, 2000);
  }

  /**
   * Log out
   * 
   * @emits Backend.EventType.logout
   * @memberOf Backend
   */
  logout() {
    this._account = null;
    this._paidUntil = null;

    // emit event
    this.emit(Backend.EventType.logout);

    // disconnect user during logout
    this.disconnect();

    // @TODO: Add logout call
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
    this.disconnect();

    this._connStatus = Backend.ConnectionState.connecting;
    this._serverAddress = addr;

    // emit: connecting
    this.emit(Backend.EventType.connecting, addr);
    
    // @TODO: Add connect call
    let timer = null;

    timer = setTimeout(() => {
      let err = null;

      // Prototype: Swedish servers will throw error during connect
      if(/se\d+\.mullvad\.net/.test(addr)) {
        err = new Error('Server is unreachable');
      }

      this._connStatus = Backend.ConnectionState.connected;

      // emit: connect
      this.emit(Backend.EventType.connect, addr, err);
      this._refreshIp();

      // reset timer
      timer = null;
      this._cancellationHandler  = null;
    }, 5000);

    this._cancellationHandler = () => {
      if(timer !== null) {
        clearTimeout(timer);
        this._timer = null;
      }
      this._cancellationHandler = null;
      this._connStatus = Backend.ConnectionState.disconnected;
    };
  }

  /**
   * Disconnect from VPN server
   * 
   * @emits Backend.EventType.disconnect
   * @memberOf Backend
   */
  disconnect() {
    if(this._connStatus === Backend.ConnectionState.disconnected) { return; }

    this._connStatus = Backend.ConnectionState.disconnected;
    this._serverAddress = null;

    // cancel ongoing connection attempt
    if(this._cancellationHandler) {
      this._cancellationHandler();
    } else {
      this._refreshIp();
    }

    // emit: disconnect
    this.emit(Backend.EventType.disconnect);

    // @TODO: Add disconnect call
  }
  
  /**
   * Fetch user location
   * 
   * @private
   * @returns {promise}
   * 
   * @memberOf Backend
   */
  _fetchLocation() {
    return fetch('https://freegeoip.net/json/').then((res) => {
      return res.json();
    });
  }

  /**
   * Request updates for user IP
   * 
   * @private
   * @emits Backend.EventType.updatedLocation
   * @emits Backend.EventType.updatedIp
   * 
   * @memberOf Backend
   */
  _refreshIp() {
    if(this._connStatus === Backend.ConnectionState.disconnected) {
      this._fetchLocation().then((res) => {
        const data = {
          location: [ res.latitude, res.longitude ], // lat, lng
          city: res.city,
          country: res.country_name
        };
        this.emit(Backend.EventType.updatedLocation, data);
        this.emit(Backend.EventType.updatedIp, res.ip);
      }).catch((error) => {
        console.log('Got error: ', error);
      });
      return;
    }

    let ip = [];
    for(let i = 0; i < 4; i++) {
      ip.push(parseInt(Math.random() * 253 + 1));
    }

    this.emit(Backend.EventType.updatedIp, ip.join('.'));
  }
}
