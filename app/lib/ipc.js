import moment from 'moment';

const mockUnsecureGeoIp = {
  ip: '192.168.1.2',
  latlong: [60,61],
  country: 'sweden',
  city: 'bollebygd',
};
const mockSecureGeoIp = {
  ip: '1.2.3.4',
  latlong: [1,2],
  country: 'Narnia',
  city: 'LE CITY',
};
export default class Ipc {

  constructor(connectionString) {
    this.connectionString = connectionString;
  }

  on(event/*, listener*/) {
    console.log('Adding a listener to', event);
  }

  send(action, data) {
    return this.mockSend(action, data)
      .catch(e => {
        console.error('IPC call failed', action, data, e);
        throw e;
      });
  }

  mockSend(action, data) {
    const actions = {
      login: this.mockLogin,
      logout: this.mockLogout,
      connect: ::this.mockConnect,
      cancelConnection: this.mockCancelConnection,
      disconnect: ::this.mockDisconnect,
      getConnectionInfo: () => {
        return new Promise((resolve) => {
          resolve({
            ip: this._mockIsConnected ? mockSecureGeoIp.ip : mockUnsecureGeoIp.ip,
          });
        });
      },
      getLocation: () => {
        return new Promise((resolve) => {
          const data = this._mockIsConnected ? mockSecureGeoIp : mockUnsecureGeoIp;
          resolve({
            latlong: data.latlong,
            country: data.country,
            city: data.city,
          });
        });
      },
    };

    const actionCb = actions[action];
    if (actionCb) {
      return actionCb(action, data);
    } else {
      console.log('UNKNOWN IPC ACTION', action);
      return new Promise((resolve, reject) => {
        reject('Unknown IPC action ' + action);
      });
    }
  }

  mockLogin(action, data) {
    return new Promise((resolve, reject) => {
      let res = {account: data.accountNumber};

      if(data.accountNumber.startsWith('1111')) { // accounts starting with 1111 expire in one month
        res.paidUntil = moment().startOf('day').add(15, 'days').toISOString();
      } else if(data.accountNumber.startsWith('2222')) { // expired in 2013
        res.paidUntil = moment('2013-01-01').toISOString();
      } else if(data.accountNumber.startsWith('3333')) { // expire in 2038
        res.paidUntil = moment('2038-01-01').toISOString();
      } else {
        reject(new Error('Invalid account number.'));
        return;
      }

      resolve(res);
    });
  }

  mockLogout() {
    return new Promise(function(resolve) {
      resolve();
    });
  }

  mockConnect(action, data) {
    return new Promise((resolve, reject) => {
      // Prototype: Swedish servers will throw error during connect
      if(/se\d+\.mullvad\.net/.test(data.address)) {
        reject(new Error('Server is unreachable'));
      } else {
        this._mockIsConnected = true;
        resolve();
      }
    });
  }

  mockCancelConnection() {
    return new Promise(function(resolve) {
      resolve();
    });
  }

  mockDisconnect() {
    return new Promise((resolve) => {
      this._mockIsConnected = false;
      resolve();
    });
  }
}
