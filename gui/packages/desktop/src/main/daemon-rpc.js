// @flow

import JsonRpcClient, {
  RemoteError as JsonRpcRemoteError,
  TimeOutError as JsonRpcTimeOutError,
  SocketTransport,
} from './jsonrpc-client';
import { CommunicationError, InvalidAccountError, NoDaemonError } from './errors';

import {
  object,
  partialObject,
  maybe,
  string,
  number,
  boolean,
  enumeration,
  arrayOf,
  oneOf,
} from 'validated/schema';
import { validate } from 'validated/object';

import type { Node as SchemaNode } from 'validated/schema';

export type AccountData = { expiry: string };
export type AccountToken = string;
export type Ip = string;
export type Location = {
  ip: ?string,
  country: string,
  city: ?string,
  latitude: number,
  longitude: number,
  mullvadExitIp: boolean,
  hostname: ?string,
};
const LocationSchema = partialObject({
  ip: maybe(string),
  country: string,
  city: maybe(string),
  latitude: number,
  longitude: number,
  mullvad_exit_ip: boolean,
  hostname: maybe(string),
});

export type BlockReason =
  | {
      reason:
        | 'ipv6_unavailable'
        | 'set_security_policy_error'
        | 'set_dns_error'
        | 'start_tunnel_error'
        | 'no_matching_relay'
        | 'is_offline'
        | 'tap_adapter_problem',
    }
  | { reason: 'auth_failed', details: ?string };

export type AfterDisconnect = 'nothing' | 'block' | 'reconnect';

export type TunnelState = 'connecting' | 'connected' | 'disconnecting' | 'disconnected' | 'blocked';

export type TunnelEndpoint = {
  address: string,
  tunnel: TunnelEndpointData,
};

export type TunnelEndpointData = {
  openvpn: {
    port: number,
    protocol: RelayProtocol,
  },
};

export type TunnelStateTransition =
  | { state: 'disconnected' }
  | { state: 'connecting', details: ?TunnelEndpoint }
  | { state: 'connected', details: TunnelEndpoint }
  | { state: 'disconnecting', details: AfterDisconnect }
  | { state: 'blocked', details: BlockReason };

export type RelayProtocol = 'tcp' | 'udp';
export type RelayLocation =
  | {| hostname: [string, string, string] |}
  | {| city: [string, string] |}
  | {| country: string |};

type OpenVpnConstraints = {
  port: 'any' | { only: number },
  protocol: 'any' | { only: RelayProtocol },
};

type TunnelConstraints<TOpenVpnConstraints> = {
  openvpn: TOpenVpnConstraints,
};

type RelaySettingsNormal<TTunnelConstraints> = {
  location:
    | 'any'
    | {
        only: RelayLocation,
      },
  tunnel:
    | 'any'
    | {
        only: TTunnelConstraints,
      },
};

// types describing the structure of RelaySettings
export type RelaySettingsCustom = {
  host: string,
  tunnel: TunnelEndpointData,
};
export type RelaySettings =
  | {|
      normal: RelaySettingsNormal<TunnelConstraints<OpenVpnConstraints>>,
    |}
  | {|
      customTunnelEndpoint: RelaySettingsCustom,
    |};

// types describing the partial update of RelaySettings
export type RelaySettingsNormalUpdate = $Shape<
  RelaySettingsNormal<TunnelConstraints<$Shape<OpenVpnConstraints>>>,
>;
export type RelaySettingsUpdate =
  | {|
      normal: RelaySettingsNormalUpdate,
    |}
  | {|
      customTunnelEndpoint: RelaySettingsCustom,
    |};

const constraint = <T>(constraintValue: SchemaNode<T>) => {
  return oneOf(
    string, // any
    object({
      only: constraintValue,
    }),
  );
};

const TunnelEndpointDataSchema = partialObject({
  openvpn: partialObject({
    port: number,
    protocol: enumeration('udp', 'tcp'),
  }),
});

const RelaySettingsSchema = oneOf(
  object({
    normal: partialObject({
      location: constraint(
        oneOf(
          object({
            hostname: arrayOf(string),
          }),
          object({
            city: arrayOf(string),
          }),
          object({
            country: string,
          }),
        ),
      ),
      tunnel: constraint(
        partialObject({
          openvpn: partialObject({
            port: constraint(number),
            protocol: constraint(enumeration('udp', 'tcp')),
          }),
        }),
      ),
    }),
  }),
  object({
    custom_tunnel_endpoint: partialObject({
      host: string,
      tunnel: TunnelEndpointDataSchema,
    }),
  }),
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
  relays: Array<RelayListHostname>,
};

export type RelayListHostname = {
  hostname: string,
  ipv4AddrIn: string,
  includeInCountry: boolean,
  weight: number,
};

const RelayListSchema = partialObject({
  countries: arrayOf(
    partialObject({
      name: string,
      code: string,
      cities: arrayOf(
        partialObject({
          name: string,
          code: string,
          latitude: number,
          longitude: number,
          relays: arrayOf(
            partialObject({
              hostname: string,
              ipv4_addr_in: string,
              include_in_country: boolean,
              weight: number,
            }),
          ),
        }),
      ),
    }),
  ),
});

export type TunnelOptions = {
  enableIpv6: boolean,
  openvpn: {
    mssfix: ?number,
  },
  proxy: ?ProxySettings,
};

export type ProxySettings = LocalProxySettings | RemoteProxySettings;

export type LocalProxySettings = {
  port: number,
  peer: string,
};

export type RemoteProxySettings = {
  address: string,
  auth: ?RemoteProxyAuth,
};

export type RemoteProxyAuth = {
  username: string,
  password: string,
};

const OpenVpnProxySchema = maybe(
  oneOf(
    object({
      local: partialObject({
        port: number,
        peer: string,
      }),
    }),
    object({
      remote: partialObject({
        address: string,
        auth: maybe(
          partialObject({
            username: string,
            password: string,
          }),
        ),
      }),
    }),
  ),
);

const TunnelOptionsSchema = partialObject({
  enable_ipv6: boolean,
  openvpn: partialObject({
    mssfix: maybe(number),
    proxy: OpenVpnProxySchema,
  }),
});

const AccountDataSchema = partialObject({
  expiry: string,
});

const TunnelStateTransitionSchema = oneOf(
  object({
    state: enumeration('disconnecting'),
    details: enumeration('nothing', 'block', 'reconnect'),
  }),
  object({
    state: enumeration('connecting', 'connected'),
    details: partialObject({
      address: string,
      tunnel: TunnelEndpointDataSchema,
    }),
  }),
  object({
    state: enumeration('blocked'),
    details: oneOf(
      object({
        reason: enumeration(
          'ipv6_unavailable',
          'set_security_policy_error',
          'set_dns_error',
          'start_tunnel_error',
          'no_matching_relay',
          'is_offline',
          'tap_adapter_problem',
        ),
      }),
      object({
        reason: enumeration('auth_failed'),
        details: maybe(string),
      }),
    ),
  }),
  object({
    state: enumeration('connected', 'connecting', 'disconnected'),
  }),
);

export type AppVersionInfo = {
  currentIsSupported: boolean,
  latest: {
    latestStable: string,
    latest: string,
  },
};

const AppVersionInfoSchema = partialObject({
  current_is_supported: boolean,
  latest: partialObject({
    latest_stable: string,
    latest: string,
  }),
});

export class ConnectionObserver {
  _openHandler: () => void;
  _closeHandler: (error: ?Error) => void;

  constructor(openHandler: () => void, closeHandler: (error: ?Error) => void) {
    this._openHandler = openHandler;
    this._closeHandler = closeHandler;
  }

  _onOpen = () => {
    this._openHandler();
  };

  _onClose = (error: ?Error) => {
    this._closeHandler(error);
  };
}

export class SubscriptionListener<T> {
  _eventHandler: (payload: T) => void;
  _errorHandler: (error: Error) => void;

  constructor(eventHandler: (payload: T) => void, errorHandler: (error: Error) => void) {
    this._eventHandler = eventHandler;
    this._errorHandler = errorHandler;
  }

  _onEvent(payload: T) {
    this._eventHandler(payload);
  }

  _onError(error: Error) {
    this._errorHandler(error);
  }
}

export type Settings = {
  accountToken: ?AccountToken,
  allowLan: boolean,
  autoConnect: boolean,
  blockWhenDisconnected: boolean,
  relaySettings: RelaySettings,
  tunnelOptions: TunnelOptions,
};

const SettingsSchema = partialObject({
  account_token: maybe(string),
  allow_lan: boolean,
  auto_connect: boolean,
  block_when_disconnected: boolean,
  relay_settings: RelaySettingsSchema,
  tunnel_options: TunnelOptionsSchema,
});

export interface DaemonRpcProtocol {
  connect({ path: string }): void;
  disconnect(): void;
  getAccountData(AccountToken): Promise<AccountData>;
  getRelayLocations(): Promise<RelayList>;
  setAccount(accountToken: ?AccountToken): Promise<void>;
  updateRelaySettings(RelaySettingsUpdate): Promise<void>;
  setAllowLan(boolean): Promise<void>;
  setEnableIpv6(boolean): Promise<void>;
  setBlockWhenDisconnected(boolean): Promise<void>;
  setOpenVpnMssfix(?number): Promise<void>;
  setAutoConnect(boolean): Promise<void>;
  connectTunnel(): Promise<void>;
  disconnectTunnel(): Promise<void>;
  getLocation(): Promise<Location>;
  getState(): Promise<TunnelStateTransition>;
  getSettings(): Promise<Settings>;
  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void>;
  subscribeSettingsListener(listener: SubscriptionListener<Settings>): Promise<void>;
  addConnectionObserver(observer: ConnectionObserver): void;
  removeConnectionObserver(observer: ConnectionObserver): void;
  getAccountHistory(): Promise<Array<AccountToken>>;
  removeAccountFromHistory(accountToken: AccountToken): Promise<void>;
  getCurrentVersion(): Promise<string>;
  getVersionInfo(): Promise<AppVersionInfo>;
}

export class ResponseParseError extends Error {
  _validationError: ?Error;

  constructor(message: string, validationError: ?Error) {
    super(message);
    this._validationError = validationError;
  }

  get validationError(): ?Error {
    return this._validationError;
  }
}

// Timeout used for RPC calls that do networking
const NETWORK_CALL_TIMEOUT = 10000;

export class DaemonRpc implements DaemonRpcProtocol {
  _transport = new JsonRpcClient(new SocketTransport());

  connect(connectionParams: { path: string }) {
    this._transport.connect(connectionParams);
  }

  disconnect() {
    this._transport.disconnect();
  }

  addConnectionObserver(observer: ConnectionObserver) {
    this._transport.on('open', observer._onOpen).on('close', observer._onClose);
  }

  removeConnectionObserver(observer: ConnectionObserver) {
    this._transport.off('open', observer._onOpen).off('close', observer._onClose);
  }

  async getAccountData(accountToken: AccountToken): Promise<AccountData> {
    let response;
    try {
      response = await this._transport.send('get_account_data', accountToken, NETWORK_CALL_TIMEOUT);
    } catch (error) {
      if (error instanceof JsonRpcRemoteError) {
        switch (error.code) {
          case -200: // Account doesn't exist
            throw new InvalidAccountError();
          case -32603: // Internal error
            throw new CommunicationError();
        }
      } else if (error instanceof JsonRpcTimeOutError) {
        throw new NoDaemonError();
      } else {
        throw error;
      }
    }

    try {
      return validate(AccountDataSchema, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_account_data', error);
    }
  }

  async getRelayLocations(): Promise<RelayList> {
    const response = await this._transport.send('get_relay_locations');
    try {
      return camelCaseObjectKeys(validate(RelayListSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_relay_locations', error);
    }
  }

  async setAccount(accountToken: ?AccountToken): Promise<void> {
    await this._transport.send('set_account', accountToken);
  }

  async updateRelaySettings(relaySettings: RelaySettingsUpdate): Promise<void> {
    await this._transport.send('update_relay_settings', [underscoreObjectKeys(relaySettings)]);
  }

  async setAllowLan(allowLan: boolean): Promise<void> {
    await this._transport.send('set_allow_lan', [allowLan]);
  }

  async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this._transport.send('set_enable_ipv6', [enableIpv6]);
  }

  async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this._transport.send('set_block_when_disconnected', [blockWhenDisconnected]);
  }

  async setOpenVpnMssfix(mssfix: ?number): Promise<void> {
    await this._transport.send('set_openvpn_mssfix', [mssfix]);
  }

  async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this._transport.send('set_auto_connect', [autoConnect]);
  }

  async connectTunnel(): Promise<void> {
    await this._transport.send('connect');
  }

  async disconnectTunnel(): Promise<void> {
    await this._transport.send('disconnect');
  }

  async getLocation(): Promise<Location> {
    const response = await this._transport.send('get_current_location', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(LocationSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_location', error);
    }
  }

  async getState(): Promise<TunnelStateTransition> {
    const response = await this._transport.send('get_state');
    try {
      return camelCaseObjectKeys(validate(TunnelStateTransitionSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_state', error);
    }
  }

  async getSettings(): Promise<Settings> {
    const response = await this._transport.send('get_settings');
    try {
      return camelCaseObjectKeys(validate(SettingsSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_settings', error);
    }
  }

  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void> {
    return this._transport.subscribe('new_state', (payload) => {
      try {
        const newState = camelCaseObjectKeys(validate(TunnelStateTransitionSchema, payload));
        listener._onEvent(newState);
      } catch (error) {
        listener._onError(new ResponseParseError('Invalid payload from new_state', error));
      }
    });
  }

  subscribeSettingsListener(listener: SubscriptionListener<Settings>): Promise<void> {
    return this._transport.subscribe('settings', (payload) => {
      try {
        const newSettings = camelCaseObjectKeys(validate(SettingsSchema, payload));
        listener._onEvent(newSettings);
      } catch (error) {
        listener._onError(new ResponseParseError('Invalid payload from settings', error));
      }
    });
  }

  async getAccountHistory(): Promise<Array<AccountToken>> {
    const response = await this._transport.send('get_account_history');
    try {
      return validate(arrayOf(string), response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_account_history', null);
    }
  }

  async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this._transport.send('remove_account_from_history', accountToken);
  }

  async getCurrentVersion(): Promise<string> {
    const response = await this._transport.send('get_current_version');
    try {
      return validate(string, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_version', null);
    }
  }

  async getVersionInfo(): Promise<AppVersionInfo> {
    const response = await this._transport.send('get_version_info', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(AppVersionInfoSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_version_info', null);
    }
  }
}

function underscoreToCamelCase(str: string): string {
  return str.replace(/_([a-z])/gi, (matches) => matches[1].toUpperCase());
}

function camelCaseToUnderscore(str: string): string {
  return str
    .replace(/[a-z0-9][A-Z]/g, (matches) => `${matches[0]}_${matches[1].toLowerCase()}`)
    .toLowerCase();
}

function camelCaseObjectKeys(object: Object) {
  return transformObjectKeys(object, underscoreToCamelCase);
}

function underscoreObjectKeys(object: Object) {
  return transformObjectKeys(object, camelCaseToUnderscore);
}

function transformObjectKeys(object: Object, keyTransformer: (string) => string) {
  for (const sourceKey of Object.keys(object)) {
    const targetKey = keyTransformer(sourceKey);
    const sourceValue = object[sourceKey];

    object[targetKey] =
      sourceValue !== null && typeof sourceValue === 'object'
        ? transformObjectKeys(sourceValue, keyTransformer)
        : sourceValue;

    if (sourceKey !== targetKey) {
      delete object[sourceKey];
    }
  }
  return object;
}
