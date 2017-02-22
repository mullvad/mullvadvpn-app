import Enum from './enum';
import { EventEmitter } from 'events';

const EventType = Enum('connect', 'connecting', 'disconnect', 'login', 'logging', 'logout');

/**
 * Backend implementation
 * 
 * @class Backend
 */
export default class Backend extends EventEmitter {
  
  static EventType = EventType;
  
  constructor() {
    super();
    this._account = null;
    this._loggedIn = false;
    this._serverAddress = null;
  }

  // Accessors

  get account() { return this._account; }
  get loggedIn() { return this._loggedIn; }
  get serverAddress() { return this._serverAddress; }

  // Public methods

  login(account) {
    this._account = account;

    // emit: logging in
    this.emit(EventType.logging, account);

    // @TODO: Add login call
    setTimeout(() => {
      let err;
      if(!account.startsWith('1111')) {
        err = new Error('Invalid account number.');
      }
      // emit: login
      this.emit(EventType.login, account, err);
    }, 2000);
  }

  logout() {
    this._account = null;

    // emit event
    this.emit(EventType.logout);

    // @TODO: Add logout call
  }

  connect(addr) {
    this._serverAddress = addr;

    // emit: connecting
    this.emit(EventType.connecting, addr);
    
    // @TODO: Add connect call
    setTimeout(() => {
      let err;
      if(!/se\d+\.mullvad\.net/.test(addr)) {
        err = new Error('Server is unreachable');
      }
      // emit: connect
      this.emit(EventType.connect, addr, err);
    }, 5000);
  }

  disconnect() {
    this._serverAddress = null;

    // emit: disconnect
    this.emit(EventType.disconnect);

    // @TODO: Add disconnect call
  }
}
