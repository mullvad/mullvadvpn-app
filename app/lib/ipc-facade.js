// @flow

import JsonRpcWs, { InvalidReply } from './jsonrpc-ws-ipc';
import { object, string, arrayOf, number } from 'validated/schema';
import { validate } from 'validated/object';
import log from 'electron-log';

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

export type IpcCredentials = {
  connectionString: string,
  sharedSecret: string,
};

export function parseIpcCredentials(data: string): ?IpcCredentials {
  const [connectionString, sharedSecret] = data.split('\n', 2);
  if(connectionString && sharedSecret) {
    return {
      connectionString,
      sharedSecret,
    };
  } else {
    return null;
  }
}


export interface IpcFacade {
  setCredentials(IpcCredentials): void,
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
}

export class RealIpc implements IpcFacade {

  _ipc: JsonRpcWs;
  _credentials: ?IpcCredentials;
  _authenticationPromise: ?Promise<void>;

  constructor(credentials: IpcCredentials) {
    this._credentials = credentials;
    this._ipc = new JsonRpcWs(credentials.connectionString);

    // force to re-authenticate when connection closed
    this._ipc.setCloseConnectionHandler(() => {
      this._authenticationPromise = null;
    });
  }

  setCredentials(credentials: IpcCredentials) {
    this._credentials = credentials;
    this._ipc.setConnectionString(credentials.connectionString);
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
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('get_account')
        .then( raw => {
          if (raw === undefined || raw === null || typeof raw === 'string') {
            return raw;
          } else {
            throw new InvalidReply(raw);
          }
        });
    });
  }

  setAccount(accountToken: ?AccountToken): Promise<void> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('set_account', accountToken)
        .then(this._ignoreResponse);
    });
  }

  _ignoreResponse(_response: mixed): void {
    return;
  }

  setCustomRelay(relayEndpoint: RelayEndpoint): Promise<void> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('set_custom_relay', [relayEndpoint])
        .then(this._ignoreResponse);
    });
  }

  connect(): Promise<void> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('connect')
        .then(this._ignoreResponse);
    });
  }

  disconnect(): Promise<void> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('disconnect')
        .then(this._ignoreResponse);
    });
  }

  getIp(): Promise<Ip> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('get_ip')
        .then(raw => {
          if (typeof raw === 'string' && raw) {
            return raw;
          } else {
            throw new InvalidReply(raw, 'Expected a string');
          }
        });
    });
  }

  getLocation(): Promise<Location> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('get_location')
        .then(raw => {
          try {
            const validated: any = validate(LocationSchema, raw);
            return (validated: Location);
          } catch (e) {
            throw new InvalidReply(raw, e);
          }
        });
    });
  }

  getState(): Promise<BackendState> {
    return this._ensureAuthenticated().then(() => {
      return this._ipc.send('get_state')
        .then(raw => {
          return this._parseBackendState(raw);
        });
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
    this._ensureAuthenticated().then(() => {
      this._ipc.on('new_state', (rawEvent) => {
        const parsedEvent : BackendState = this._parseBackendState(rawEvent);

        listener(parsedEvent);
      });
    });
  }

  _ensureAuthenticated(): Promise<void> {
    if(this._credentials) {
      const credentials = this._credentials;
      if(!this._authenticationPromise) {
        this._authenticationPromise = this._authenticate(credentials.sharedSecret);
      }
      return this._authenticationPromise;
    } else {
      return Promise.reject(new Error('Missing authentication credentials.'));
    }
  }

  _authenticate(sharedSecret: string): Promise<void> {
    return this._ipc.send('auth', sharedSecret)
      .then(() => {
        log.info('Authenticated with backend');
      })
      .catch((e) => {
        log.error('Failed to authenticate with backend: ', e.message);
        throw e;
      });
  }
}
