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
    return new Promise((resolve, reject) => {
      this._account = account;

      // emit: logging in
      this.emit(EventType.logging, account);

      // @TODO: Add login call
      setTimeout(() => {
        if(account.startsWith('1111')) {
          // emit: login
          this.emit(EventType.login, account);

          resolve(true);
        } else {
          const err = new Error('Invalid account number.');

          // emit: login
          this.emit(EventType.login, account, err);

          reject(err);
        }
      }, 2000);
    });
  }

  logout() {
    return new Promise((resolve, reject) => {
      this._account = null;

      // emit event
      this.emit(EventType.logout);

      // @TODO: Add logout call
      resolve();
    });
  }

  connect(addr) {
    return new Promise((resolve, reject) => {
      this._serverAddress = addr;
      
      // @TODO: Add connect call
      setTimeout(() => {
        if(/se\d+\.mullvad\.net/.test(addr)) {

          // emit: connect
          this.emit(EventType.connect, addr, err);
          
          resolve(true);
        } else {
          const err = new Error('Server is unreachable');

          // emit: connect
          this.emit(EventType.connect, addr, err);

          reject(err);
        }
      }, 2000);
    });
  }

  disconnect() {
    return new Promise((resolve, reject) => {
      this._serverAddress = null;

      // emit: disconnect
      this.emit(EventType.disconnect);

      // @TODO: Add disconnect call
      resolve();
    });
  }
}
