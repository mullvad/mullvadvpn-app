// @flow

import JsonRpcWs from './jsonrpc-ws-ipc';

export type AccountData = {paid_until: string};
export type AccountNumber = string;
export type Ip = string;
export type Location = {
  latlong: Array<Number>,
  country: string,
  city: string,
};

export interface IpcFacade {
  getAccountData(AccountNumber): Promise<AccountData>,
  setAccount(accountNumber: AccountNumber): Promise<void>,
  setCountry(address: string): Promise<void>,
  connect(): Promise<void>,
  disconnect(): Promise<void>,
  getIp(): Promise<Ip>,
  getLocation(): Promise<Location>,
}

export class RealIpc implements IpcFacade {

  _ipc: JsonRpcWs;

  constructor(connectionString: ?string) {
    this._ipc = new JsonRpcWs(connectionString);
  }

  getAccountData(accountNumber: AccountNumber): Promise<AccountData> {
    return this._ipc.send('get_account_data', accountNumber)
      .then(raw => {
        // TODO: Validate here
        return raw;
      });
  }

  setAccount(accountNumber: AccountNumber): Promise<void> {
    return this._ipc.send('set_account', accountNumber);
  }

  setCountry(address: string): Promise<void> {
    return this._ipc.send('set_country', address);
  }

  connect(): Promise<void> {
    return this._ipc.send('connect');
  }

  disconnect(): Promise<void> {
    return this._ipc.send('disconnect');
  }

  getIp(): Promise<Ip> {
    return this._ipc.send('get_ip')
      .then(raw => {
        // TODO: Validate here
        return raw;
      });
  }

  getLocation(): Promise<Location> {
    return this._ipc.send('get_location')
      .then(raw => {
        // TODO: Validate here
        return raw;
      });
  }
}
