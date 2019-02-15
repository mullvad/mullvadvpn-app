import {
  AccountToken,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  ISettings,
  RelaySettingsUpdate,
  TunnelStateTransition,
} from '../shared/daemon-rpc-types';
import { CommunicationError, InvalidAccountError, NoDaemonError } from './errors';
import JsonRpcClient, {
  RemoteError as JsonRpcRemoteError,
  SocketTransport,
  TimeOutError as JsonRpcTimeOutError,
} from './jsonrpc-client';

import { validate } from 'validated/object';
import {
  arrayOf,
  boolean,
  enumeration,
  maybe,
  Node as SchemaNode,
  number,
  object,
  oneOf,
  partialObject,
  string,
} from 'validated/schema';

const locationSchema = maybe(
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

const customTunnelEndpointSchema = oneOf(
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

const relaySettingsSchema = oneOf(
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
        oneOf(
          object({
            openvpn: partialObject({
              port: constraint(number),
              protocol: constraint(enumeration('udp', 'tcp')),
            }),
          }),
          object({
            wireguard: partialObject({
              port: constraint(number),
            }),
          }),
        ),
      ),
    }),
  }),
  object({
    custom_tunnel_endpoint: partialObject({
      host: string,
      config: customTunnelEndpointSchema,
    }),
  }),
);

const relayListSchema = partialObject({
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

const openVpnProxySchema = maybe(
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
    object({
      shadowsocks: partialObject({
        peer: string,
        password: string,
        cipher: string,
      }),
    }),
  ),
);

const tunnelOptionsSchema = partialObject({
  openvpn: partialObject({
    mssfix: maybe(number),
    proxy: openVpnProxySchema,
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

const accountDataSchema = partialObject({
  expiry: string,
});

const tunnelStateTransitionSchema = oneOf(
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

const appVersionInfoSchema = partialObject({
  current_is_supported: boolean,
  latest: partialObject({
    latest_stable: string,
    latest: string,
  }),
});

export class ConnectionObserver {
  constructor(private openHandler: () => void, private closeHandler: (error?: Error) => void) {}

  // Only meant to be called by DaemonRpc
  // @internal
  public onOpen = () => {
    this.openHandler();
  };

  // Only meant to be called by DaemonRpc
  // @internal
  public onClose = (error?: Error) => {
    this.closeHandler(error);
  };
}

export class SubscriptionListener<T> {
  constructor(
    private eventHandler: (payload: T) => void,
    private errorHandler: (error: Error) => void,
  ) {}

  // Only meant to be called by DaemonRpc
  // @internal
  public onEvent(payload: T) {
    this.eventHandler(payload);
  }

  // Only meant to be called by DaemonRpc
  // @internal
  public onError(error: Error) {
    this.errorHandler(error);
  }
}

const settingsSchema = partialObject({
  account_token: maybe(string),
  allow_lan: boolean,
  auto_connect: boolean,
  block_when_disconnected: boolean,
  relay_settings: relaySettingsSchema,
  tunnel_options: tunnelOptionsSchema,
});

export class ResponseParseError extends Error {
  constructor(message: string, private validationErrorValue?: Error) {
    super(message);
  }

  get validationError(): Error | undefined {
    return this.validationErrorValue;
  }
}

// Timeout used for RPC calls that do networking
const NETWORK_CALL_TIMEOUT = 10000;

export class DaemonRpc {
  private transport = new JsonRpcClient(new SocketTransport());

  public connect(connectionParams: { path: string }) {
    this.transport.connect(connectionParams);
  }

  public disconnect() {
    this.transport.disconnect();
  }

  public addConnectionObserver(observer: ConnectionObserver) {
    this.transport.on('open', observer.onOpen).on('close', observer.onClose);
  }

  public removeConnectionObserver(observer: ConnectionObserver) {
    this.transport.off('open', observer.onOpen).off('close', observer.onClose);
  }

  public async getAccountData(accountToken: AccountToken): Promise<IAccountData> {
    let response;
    try {
      response = await this.transport.send('get_account_data', accountToken, NETWORK_CALL_TIMEOUT);
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
      return validate(accountDataSchema, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_account_data', error);
    }
  }

  public async getRelayLocations(): Promise<IRelayList> {
    const response = await this.transport.send('get_relay_locations');
    try {
      return camelCaseObjectKeys(validate(relayListSchema, response)) as IRelayList;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_relay_locations', error);
    }
  }

  public async setAccount(accountToken?: AccountToken): Promise<void> {
    await this.transport.send('set_account', [accountToken]);
  }

  public async updateRelaySettings(relaySettings: RelaySettingsUpdate): Promise<void> {
    await this.transport.send('update_relay_settings', [underscoreObjectKeys(relaySettings)]);
  }

  public async setAllowLan(allowLan: boolean): Promise<void> {
    await this.transport.send('set_allow_lan', [allowLan]);
  }

  public async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this.transport.send('set_enable_ipv6', [enableIpv6]);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this.transport.send('set_block_when_disconnected', [blockWhenDisconnected]);
  }

  public async setOpenVpnMssfix(mssfix?: number): Promise<void> {
    await this.transport.send('set_openvpn_mssfix', [mssfix]);
  }

  public async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this.transport.send('set_auto_connect', [autoConnect]);
  }

  public async connectTunnel(): Promise<void> {
    await this.transport.send('connect');
  }

  public async disconnectTunnel(): Promise<void> {
    await this.transport.send('disconnect');
  }

  public async getLocation(): Promise<ILocation | undefined> {
    const response = await this.transport.send('get_current_location', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(locationSchema, response)) as ILocation;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_location', error);
    }
  }

  public async getState(): Promise<TunnelStateTransition> {
    const response = await this.transport.send('get_state');
    try {
      return camelCaseObjectKeys(
        validate(tunnelStateTransitionSchema, response),
      ) as TunnelStateTransition;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_state', error);
    }
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.transport.send('get_settings');
    try {
      return camelCaseObjectKeys(validate(settingsSchema, response)) as ISettings;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_settings', error);
    }
  }

  public subscribeStateListener(
    listener: SubscriptionListener<TunnelStateTransition>,
  ): Promise<void> {
    return this.transport.subscribe('new_state', (payload) => {
      try {
        const newState = camelCaseObjectKeys(
          validate(tunnelStateTransitionSchema, payload),
        ) as TunnelStateTransition;
        listener.onEvent(newState);
      } catch (error) {
        listener.onError(new ResponseParseError('Invalid payload from new_state', error));
      }
    });
  }

  public subscribeSettingsListener(listener: SubscriptionListener<ISettings>): Promise<void> {
    return this.transport.subscribe('settings', (payload) => {
      try {
        const newSettings = camelCaseObjectKeys(validate(settingsSchema, payload)) as ISettings;
        listener.onEvent(newSettings);
      } catch (error) {
        listener.onError(new ResponseParseError('Invalid payload from settings', error));
      }
    });
  }

  public async getAccountHistory(): Promise<AccountToken[]> {
    const response = await this.transport.send('get_account_history');
    try {
      return validate(arrayOf(string), response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_account_history');
    }
  }

  public async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this.transport.send('remove_account_from_history', accountToken);
  }

  public async getCurrentVersion(): Promise<string> {
    const response = await this.transport.send('get_current_version');
    try {
      return validate(string, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_version');
    }
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    const response = await this.transport.send('get_version_info', [], NETWORK_CALL_TIMEOUT);
    try {
      return camelCaseObjectKeys(validate(appVersionInfoSchema, response)) as IAppVersionInfo;
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

function camelCaseObjectKeys(anObject: { [key: string]: any }) {
  return transformObjectKeys(anObject, underscoreToCamelCase);
}

function underscoreObjectKeys(anObject: { [key: string]: any }) {
  return transformObjectKeys(anObject, camelCaseToUnderscore);
}

function transformObjectKeys(
  anObject: { [key: string]: any },
  keyTransformer: (key: string) => string,
) {
  for (const sourceKey of Object.keys(anObject)) {
    const targetKey = keyTransformer(sourceKey);
    const sourceValue = anObject[sourceKey];

    anObject[targetKey] =
      sourceValue !== null && typeof sourceValue === 'object'
        ? transformObjectKeys(sourceValue, keyTransformer)
        : sourceValue;

    if (sourceKey !== targetKey) {
      delete anObject[sourceKey];
    }
  }
  return anObject;
}
