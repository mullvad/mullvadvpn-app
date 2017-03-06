import moment from 'moment';
import Enum from './enum';
import { EventEmitter } from 'events';
import { servers } from '../config';
import { ConnectionState as ReduxConnectionState } from '../enums';

const EventType = Enum('connect', 'connecting', 'disconnect', 'login', 'logging', 'logout', 'updatedIp');
const ConnectionState = Enum('disconnected', 'connecting', 'connected');

/**
 * Backend implementation
 * 
 * @class Backend
 */
export default class Backend extends EventEmitter {
  
  static EventType = EventType;
  static ConnectionState = ConnectionState;
  
  constructor() {
    super();
    this._account = null;
    this._paidUntil = null;
    this._serverAddress = null;
    this._connStatus = ConnectionState.disconnected;
    this._cancellationHandler = null;

    // update IP in background
    setTimeout(::this.refreshIp, 0);
  }

  // Accessors

  get account() { return this._account; }
  get paidUntil() { return this._paidUntil; }
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
   */
  syncWithReduxStore(store) {
    const mapConnStatus = (s) => {
      const S = ReduxConnectionState;
      const BS = ConnectionState;
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

  serverInfo(key) {
    switch(key) {
    case 'fastest': return this.fastestServer();
    case 'nearest': return this.nearestServer();
    default: return servers[key];
    }
  }

  fastestServer() {
    return {
      address: 'uk.mullvad.net', 
      name: 'Fastest',
      city: 'London',
      country: 'United Kingdom',
      location: [51.5073509, -0.1277583]
    };
  }

  nearestServer() {
    return {
      address: 'es.mullvad.net', 
      name: 'Nearest',
      city: 'Madrid',
      country: 'Spain',
      location: [40.4167754, -3.7037902]
    };
  }

  login(account) {
    this._account = account;
    this._paidUntil = null;

    // emit: logging in
    this.emit(EventType.logging, { account, paidUntil: this._paidUntil }, null);

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
      this.emit(EventType.login, res, err);
    }, 2000);
  }

  logout() {
    this._account = null;
    this._paidUntil = null;

    // emit event
    this.emit(EventType.logout);

    // disconnect user during logout
    this.disconnect();

    // @TODO: Add logout call
  }

  connect(addr) {
    this.disconnect();

    this._connStatus = ConnectionState.connecting;
    this._serverAddress = addr;

    // emit: connecting
    this.emit(EventType.connecting, addr);
    
    // @TODO: Add connect call
    let timer = null;

    timer = setTimeout(() => {
      let err = null;

      // Prototype: Swedish servers will throw error during connect
      if(/se\d+\.mullvad\.net/.test(addr)) {
        err = new Error('Server is unreachable');
      }

      this._connStatus = ConnectionState.connected;

      // emit: connect
      this.emit(EventType.connect, addr, err);
      this.refreshIp();

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
      this._connStatus = ConnectionState.disconnected;
    };
  }

  disconnect() {
    if(this._connStatus === ConnectionState.disconnected) { return; }

    this._connStatus = ConnectionState.disconnected;
    this._serverAddress = null;

    // cancel ongoing connection attempt
    if(this._cancellationHandler) {
      this._cancellationHandler();
    } else {
      this.refreshIp();
    }

    // emit: disconnect
    this.emit(EventType.disconnect);

    // @TODO: Add disconnect call
  }

  refreshIp() {
    let ip = [];
    for(let i = 0; i < 4; i++) {
      ip.push(parseInt(Math.random() * 253 + 1));
    }
    this.emit(EventType.updatedIp, ip.join('.'));
  }
}
