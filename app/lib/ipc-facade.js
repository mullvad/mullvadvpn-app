// @flow

import JsonRpcWs, { InvalidReply } from './jsonrpc-ws-ipc';
import { object, string, number, boolean, enumeration, arrayOf, oneOf } from 'validated/schema';
import { validate } from 'validated/object';

export type AccountData = { expiry: string };
export type AccountToken = string;
export type Ip = string;
export type Location = {
  ip: Ip,
  country: string,
  city: string,
  latitude: number,
  longitude: number,
  mullvad_exit_ip: boolean,
};
const LocationSchema = object({
  ip: string,
  country: string,
  city: string,
  latitude: number,
  longitude: number,
  mullvad_exit_ip: boolean,
});

export type SecurityState = 'secured' | 'unsecured';
export type BackendState = {
  state: SecurityState,
  target_state: SecurityState,
};

export type RelayProtocol = 'tcp' | 'udp';
export type RelayLocation = {| city: [string, string] |} | {| country: string |};

type OpenVpnParameters = {
  port: 'any' | { only: number },
  protocol: 'any' | { only: RelayProtocol },
};

type TunnelOptions<TOpenVpnParameters> = {
  openvpn: TOpenVpnParameters,
};

type RelaySettingsNormal<TTunnelOptions> = {
  location: 'any' | {
    only: RelayLocation,
  },
  tunnel: 'any' | {
    only: TTunnelOptions
  },
};

// types describing the structure of RelaySettings
export type RelaySettingsCustom = {
  host: string,
  tunnel: {
    openvpn: {
      port: number,
      protocol: RelayProtocol
    }
  }
};
export type RelaySettings = {|
  normal: RelaySettingsNormal<TunnelOptions<OpenVpnParameters>>
|} | {|
  custom_tunnel_endpoint: RelaySettingsCustom
|};

// types describing the partial update of RelaySettings
export type RelaySettingsNormalUpdate = $Shape< RelaySettingsNormal< TunnelOptions<$Shape<OpenVpnParameters> > > >;
export type RelaySettingsUpdate = {|
  normal: RelaySettingsNormalUpdate
|} | {|
  custom_tunnel_endpoint: RelaySettingsCustom
|};

const Constraint = (v) => oneOf(string, object({
  only: v,
}));
const RelaySettingsSchema = oneOf(
  object({
    normal: object({
      location: Constraint(oneOf(
        object({
          city: arrayOf(string),
        }),
        object({
          country: string
        }),
      )),
      tunnel: Constraint(object({
        openvpn: object({
          port: Constraint(number),
          protocol: Constraint(enumeration('udp', 'tcp')),
        }),
      })),
    })
  }),
  object({
    custom_tunnel_endpoint: object({
      host: string,
      tunnel: object({
        openvpn: object({
          port: number,
          protocol: enumeration('udp', 'tcp'),
        })
      })
    })
  })
);

export type RelayList = {
  countries: Array<RelayListCountry>,
};

export type RelayListCountry = {
  name: string,
  code: string,
  cities: Array<RelayListCity>,
};

export type RelayListCity = {
  name: string,
  code: string,
  latitude: number,
  longitude: number,
  has_active_relays: boolean,
};

const RelayListSchema = object({
  countries: arrayOf(object({
    name: string,
    code: string,
    cities: arrayOf(object({
      name: string,
      code: string,
      latitude: number,
      longitude: number,
      has_active_relays: boolean,
    })),
  })),
});


export interface IpcFacade {
  setConnectionString(string): void,
  getAccountData(AccountToken): Promise<AccountData>,
  getRelayLocations(): Promise<RelayList>,
  getAccount(): Promise<?AccountToken>,
  setAccount(accountToken: ?AccountToken): Promise<void>,
  updateRelaySettings(RelaySettingsUpdate): Promise<void>,
  getRelaySettings(): Promise<RelaySettings>,
  setAllowLan(boolean): Promise<void>,
  getAllowLan(): Promise<boolean>,
  connect(): Promise<void>,
  disconnect(): Promise<void>,
  shutdown(): Promise<void>,
  getLocation(): Promise<Location>,
  getState(): Promise<BackendState>,
  registerStateListener((BackendState) => void): void,
  setCloseConnectionHandler(() => void): void,
  authenticate(sharedSecret: string): Promise<void>,
  getAccountHistory(): Promise<Array<AccountToken>>,
  removeAccountFromHistory(accountToken: AccountToken): Promise<void>,
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

  async getRelayLocations(): Promise<RelayList> {
    const raw = await this._ipc.send('get_relay_locations');
    try {
      const validated: any = validate(RelayListSchema, raw);
      return (validated: RelayList);
    } catch (e) {
      throw new InvalidReply(raw, e);
    }
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

  updateRelaySettings(relaySettings: RelaySettingsUpdate): Promise<void> {
    return this._ipc.send('update_relay_settings', [relaySettings])
      .then(this._ignoreResponse);
  }

  getRelaySettings(): Promise<RelaySettings> {
    return this._ipc.send('get_relay_settings')
      .then( raw => {
        try {
          const validated: any = validate(RelaySettingsSchema, raw);
          return (validated: RelaySettings);
        } catch (e) {
          throw new InvalidReply(raw, e);
        }
      });
  }

  setAllowLan(allowLan: boolean): Promise<void> {
    return this._ipc.send('set_allow_lan', [allowLan])
      .then(this._ignoreResponse);
  }

  async getAllowLan(): Promise<boolean> {
    const raw = await this._ipc.send('get_allow_lan');
    if(typeof(raw) === 'boolean') {
      return raw;
    } else {
      throw new InvalidReply(raw, 'Expected a boolean');
    }
  }

  connect(): Promise<void> {
    return this._ipc.send('connect')
      .then(this._ignoreResponse);
  }

  disconnect(): Promise<void> {
    return this._ipc.send('disconnect')
      .then(this._ignoreResponse);
  }

  shutdown(): Promise<void> {
    return this._ipc.send('shutdown')
      .then(this._ignoreResponse);
  }

  getLocation(): Promise<Location> {
    return this._ipc.send('get_current_location')
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

  setCloseConnectionHandler(handler: () => void) {
    this._ipc.setCloseConnectionHandler(handler);
  }

  authenticate(sharedSecret: string): Promise<void> {
    return this._ipc.send('auth', sharedSecret)
      .then(this._ignoreResponse);
  }

  getAccountHistory(): Promise<Array<AccountToken>> {
    return this._ipc.send('get_account_history')
      .then(raw => {
        if(Array.isArray(raw) && raw.every(i => typeof i === 'string')) {
          const checked: any = raw;
          return (checked: Array<AccountToken>);
        } else {
          throw new InvalidReply(raw, 'Expected an array of strings');
        }
      });
  }

  removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    return this._ipc.send('remove_account_from_history', accountToken)
      .then(this._ignoreResponse);
  }
}
