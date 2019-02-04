import JsonRpcClient, {
  RemoteError as JsonRpcRemoteError,
  TimeOutError as JsonRpcTimeOutError,
  SocketTransport,
} from './jsonrpc-client';
import { CommunicationError, InvalidAccountError, NoDaemonError } from './errors';
import {
  AccountData,
  AccountToken,
  AppVersionInfo,
  Location,
  RelayList,
  RelaySettingsUpdate,
  Settings,
  TunnelStateTransition,
} from '../shared/daemon-rpc-types';

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

import { Node as SchemaNode } from 'validated/schema';

const LocationSchema = maybe(
  partialObject({
    ip: maybe(string),
    country: string,
    city: maybe(string),
    latitude: number,
    longitude: number,
    mullvad_exit_ip: boolean,
    hostname: maybe(string),
  }),
);

const constraint = <T>(constraintValue: SchemaNode<T>) => {
  return oneOf(
    string, // any
    object({
      only: constraintValue,
    }),
  );
};

const CustomTunnelEndpoint = oneOf(
  object({
    openvpn: object({
      endpoint: object({
        address: string,
        protocol: enumeration('udp', 'tcp'),
      }),
      username: string,
      password: string,
    }),
  }),
  object({
    wireguard: object({
      tunnel: object({
        private_key: string,
        addresses: arrayOf(string),
      }),
      peer: object({
        public_key: string,
        allowed_ips: arrayOf(string),
        endpoint: string,
      }),
      gateway: string,
    }),
  }),
);

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
      config: CustomTunnelEndpoint,
    }),
  }),
);

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
  openvpn: partialObject({
    mssfix: maybe(number),
    proxy: OpenVpnProxySchema,
  }),
  wireguard: partialObject({
    mtu: maybe(number),
    // only relevant on linux
    fmwark: maybe(number),
  }),
  generic: partialObject({
    enable_ipv6: boolean,
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
      protocol: enumeration('tcp', 'udp'),
      tunnel_type: enumeration('wireguard', 'openvpn'),
    }),
  }),
  object({
    state: enumeration('blocked'),
    details: oneOf(
      object({
        reason: enumeration(
          'ipv6_unavailable',
          'set_firewall_policy_error',
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

const AppVersionInfoSchema = partialObject({
  current_is_supported: boolean,
  latest: partialObject({
    latest_stable: string,
    latest: string,
  }),
});

export class ConnectionObserver {
  _openHandler: () => void;
  _closeHandler: (error?: Error) => void;

  constructor(openHandler: () => void, closeHandler: (error?: Error) => void) {
    this._openHandler = openHandler;
    this._closeHandler = closeHandler;
  }

  _onOpen = () => {
    this._openHandler();
  };

  _onClose = (error?: Error) => {
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

const SettingsSchema = partialObject({
  account_token: maybe(string),
  allow_lan: boolean,
  auto_connect: boolean,
  block_when_disconnected: boolean,
  relay_settings: RelaySettingsSchema,
  tunnel_options: TunnelOptionsSchema,
});

export class ResponseParseError extends Error {
  _validationError?: Error;

  constructor(message: string, validationError?: Error) {
    super(message);
    this._validationError = validationError;
  }

  get validationError(): Error | undefined {
    return this._validationError;
  }
}

// Timeout used for RPC calls that do networking
const NETWORK_CALL_TIMEOUT = 10000;

export class DaemonRpc {
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
      return camelCaseObjectKeys(validate(RelayListSchema, response)) as RelayList;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_relay_locations', error);
    }
  }

  async setAccount(accountToken?: AccountToken): Promise<void> {
    await this._transport.send('set_account', [accountToken]);
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

  async setOpenVpnMssfix(mssfix?: number): Promise<void> {
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

  async getLocation(): Promise<Location | undefined> {
    const response = await this._transport.send('get_current_location', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(LocationSchema, response)) as Location;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_location', error);
    }
  }

  async getState(): Promise<TunnelStateTransition> {
    const response = await this._transport.send('get_state');
    try {
      return camelCaseObjectKeys(
        validate(TunnelStateTransitionSchema, response),
      ) as TunnelStateTransition;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_state', error);
    }
  }

  async getSettings(): Promise<Settings> {
    const response = await this._transport.send('get_settings');
    try {
      return camelCaseObjectKeys(validate(SettingsSchema, response)) as Settings;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_settings', error);
    }
  }

  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void> {
    return this._transport.subscribe('new_state', (payload) => {
      try {
        const newState = camelCaseObjectKeys(
          validate(TunnelStateTransitionSchema, payload),
        ) as TunnelStateTransition;
        listener._onEvent(newState);
      } catch (error) {
        listener._onError(new ResponseParseError('Invalid payload from new_state', error));
      }
    });
  }

  subscribeSettingsListener(listener: SubscriptionListener<Settings>): Promise<void> {
    return this._transport.subscribe('settings', (payload) => {
      try {
        const newSettings = camelCaseObjectKeys(validate(SettingsSchema, payload)) as Settings;
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
      throw new ResponseParseError('Invalid response from get_account_history');
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
      throw new ResponseParseError('Invalid response from get_current_version');
    }
  }

  async getVersionInfo(): Promise<AppVersionInfo> {
    const response = await this._transport.send('get_version_info', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(AppVersionInfoSchema, response)) as AppVersionInfo;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_version_info');
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

function camelCaseObjectKeys(object: { [key: string]: any }) {
  return transformObjectKeys(object, underscoreToCamelCase);
}

function underscoreObjectKeys(object: { [key: string]: any }) {
  return transformObjectKeys(object, camelCaseToUnderscore);
}

function transformObjectKeys(
  object: { [key: string]: any },
  keyTransformer: (key: string) => string,
) {
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
