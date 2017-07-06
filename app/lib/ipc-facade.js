// @flow

import JsonRpcWs, { InvalidReply } from './jsonrpc-ws-ipc';
import { object, string, arrayOf, number } from 'validated/schema';
import { validate } from 'validated/object';

import type { Coordinate2d } from '../types';

export type AccountData = {paid_until: string};
export type AccountNumber = string;
export type Ip = string;
export type Location = {
  latlong: Coordinate2d,
  country: string,
  city: string,
};
const LocationSchema = object({
  latlong: arrayOf(number),
  country: string,
  city: string,
});

export type BackendState = 'secured' | 'unsecured';

export interface IpcFacade {
  getAccountData(AccountNumber): Promise<AccountData>,
  setAccount(accountNumber: AccountNumber): Promise<void>,
  setCountry(address: string): Promise<void>,
  connect(): Promise<void>,
  disconnect(): Promise<void>,
  getIp(): Promise<Ip>,
  getLocation(): Promise<Location>,
  getState(): Promise<BackendState>,
  registerStateListener((BackendState) => void): void,
}

export class RealIpc implements IpcFacade {

  _ipc: JsonRpcWs;

  constructor(connectionString: string) {
    this._ipc = new JsonRpcWs(connectionString);
  }

  getAccountData(accountNumber: AccountNumber): Promise<AccountData> {
    return this._ipc.send('get_account_data', accountNumber)
      .then(raw => {
        if (typeof raw === 'object' && raw && raw.paid_until) {
          return raw;
        } else {
          throw new InvalidReply(raw, 'Expected an object with paid_until');
        }
      });
  }

  setAccount(accountNumber: AccountNumber): Promise<void> {
    return this._ipc.send('set_account', accountNumber)
      .then(this._ignoreResponse);
  }

  _ignoreResponse(_response: mixed): void {
    return;
  }

  setCountry(address: string): Promise<void> {
    return this._ipc.send('set_country', address)
      .then(this._ignoreResponse);
  }

  connect(): Promise<void> {
    return this._ipc.send('connect')
      .then(this._ignoreResponse);
  }

  disconnect(): Promise<void> {
    return this._ipc.send('disconnect')
      .then(this._ignoreResponse);
  }

  getIp(): Promise<Ip> {
    return this._ipc.send('get_ip')
      .then(raw => {
        if (typeof raw === 'string' && raw) {
          return raw;
        } else {
          throw new InvalidReply(raw, 'Expected a string');
        }
      });
  }

  getLocation(): Promise<Location> {
    return this._ipc.send('get_location')
      .then(raw => {
        try {
          const validated: any = validate(LocationSchema, raw);
          return (validated: Location);
        } catch (e) {
          throw new InvalidReply(raw, e);
        }
      });
  }

  getState(): Promise<BackendState> {
    return this._ipc.send('get_state')
      .then(raw => {
        return this._parseBackendState(raw);
      });
  }

  _parseBackendState(raw: mixed): BackendState {
    if (raw === 'secured' || raw === 'unsecured') {
      return raw;
    } else {
      throw new InvalidReply(raw);
    }
  }

  registerStateListener(listener: (BackendState) => void) {
    this._ipc.on('new_state', (rawEvent) => {
      const parsedEvent : BackendState = this._parseBackendState(rawEvent);

      listener(parsedEvent);
    });
  }
}
