import Enum from './enum';
import { EventEmitter } from 'events';

const EventType = Enum('connect', 'connecting', 'disconnect', 'login', 'logging', 'logout', 'updatedIp');

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
    this._cancellationHandler = null;

    // update IP in background
    setTimeout(::this.refreshIp, 0);
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
    let timer = null;

    timer = setTimeout(() => {
      let err;
      if(!/se\d+\.mullvad\.net/.test(addr)) {
        err = new Error('Server is unreachable');
      }
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
    }
  }

  disconnect() {
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
      ip.push(parseInt(Math.random() * 254));
    }
    this.emit(EventType.updatedIp, ip.join('.'));
  }
}
