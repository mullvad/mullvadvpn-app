import privateData from './private';

/**
 * Private data
 */
const { get: getImpl, set: setImpl } = privateData();

/**
 * Remote backend implementation
 * 
 * @class BackendImpl
 */
class BackendImpl {
  constructor() {
    this._account = null;
    this._loggedIn = false;
  }

  get account() {
    return this._account;
  }

  get loggedIn() {
    return this._loggedIn;
  }

  login(account) {
    return new Promise((resolve, reject) => {
      // @TODO: Add login call
      setTimeout(() => {
        if(account.startsWith('1111')) {
          resolve(true);
        } else {
          reject(new Error("Invalid account number."));
        }
      }, 2000);
    });
  }
}

/**
 * Backend implementation
 * 
 * @export
 * @class Backend
 */
export default class Backend {

  constructor() {
    setImpl(this, new BackendImpl());
  }

  get account() { 
    return getImpl(this).account; 
  }

  get loggedIn() { 
    return getImpl(this).loggedIn;
  }

  login(account) {
    return getImpl(this).login(account);
  }

};
