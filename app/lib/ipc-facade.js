// @flow

import JsonRpcWs, { InvalidReply } from './jsonrpc-ws-ipc';
import { object, string, arrayOf, number } from 'validated/schema';
import { validate } from 'validated/object';

import type { Coordinate2d } from '../types';

export type AccountData = {expiry: string};
export type AccountToken = string;
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

export type SecurityState = 'secured' | 'unsecured';
export type BackendState = {
  state: SecurityState,
  target_state: SecurityState,
};
export type RelayEndpoint = {
  host: string,
  port: number,
  protocol: 'tcp' | 'udp',
};


export interface IpcFacade {
  setConnectionString(string): void,
  getAccountData(AccountToken): Promise<AccountData>,
  getAccount(): Promise<?AccountToken>,
  setAccount(accountToken: ?AccountToken): Promise<void>,
  setCustomRelay(RelayEndpoint): Promise<void>,
  connect(): Promise<void>,
  disconnect(): Promise<void>,
  getIp(): Promise<Ip>,
  getLocation(): Promise<Location>,
  getState(): Promise<BackendState>,
  registerStateListener((BackendState) => void): void,
  setCloseConnectionHandler(() => void): void,
  authenticate(sharedSecret: string): Promise<void>,
}

export class RealIpc implements IpcFacade {

  _ipc: JsonRpcWs;

  constructor(connectionString: string) {
    this._ipc = new JsonRpcWs(connectionString);
  }

  setConnectionString(str: string) {
    this._ipc.setConnectionString(str);
  }

  getAccountData(accountToken: AccountToken): Promise<AccountData> {
    // send the IPC with 30s timeout since the backend will wait
    // for a HTTP request before replying

    return this._ipc.send('get_account_data', accountToken, 30000)
      .then(raw => {
        if (typeof raw === 'object' && raw && raw.expiry) {
          return raw;
        } else {
          throw new InvalidReply(raw, 'Expected an object with expiry');
        }
      });
  }

  getAccount(): Promise<?AccountToken> {
    return this._ipc.send('get_account')
      .then( raw => {
        if (raw === undefined || raw === null || typeof raw === 'string') {
          return raw;
        } else {
          throw new InvalidReply(raw);
        }
      });
  }

  setAccount(accountToken: ?AccountToken): Promise<void> {
    return this._ipc.send('set_account', accountToken)
      .then(this._ignoreResponse);
  }

  _ignoreResponse(_response: mixed): void {
    return;
  }

  setCustomRelay(relayEndpoint: RelayEndpoint): Promise<void> {
    return this._ipc.send('set_custom_relay', [relayEndpoint])
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
    if (raw && raw.state && raw.target_state) {

      const uncheckedRaw: any = raw;

      const states: Array<SecurityState> = ['secured', 'unsecured'];
      const correctState = states.includes(uncheckedRaw.state);
      const correctTargetState = states.includes(uncheckedRaw.target_state);

      if (!correctState || !correctTargetState) {
        throw new InvalidReply(raw);
      }

      return (uncheckedRaw: BackendState);
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

  authenticate(sharedSecret: string): Promise<void> {
    return this._ipc.send('auth', sharedSecret)
      .then(this._ignoreResponse);
  }

  setCloseConnectionHandler(handler: () => void) {
    console.log('appa', handler);
  }
}
