import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  DaemonEvent,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  ISettings,
  IWireguardPublicKey,
  KeygenEvent,
  RelaySettingsUpdate,
  TunnelState,
  VoucherErrorCode,
  VoucherResponse,
} from '../shared/daemon-rpc-types';
import { CommunicationError, InvalidAccountError, NoDaemonError } from './errors';
import JsonRpcClient, {
  RemoteError as JsonRpcRemoteError,
  SocketTransport,
  TimeOutError as JsonRpcTimeOutError,
} from './jsonrpc-client';
import { camelCaseToSnakeCase, snakeCaseToCamelCase } from './transform-object-keys';

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
    ipv4: maybe(string),
    ipv6: maybe(string),
    country: string,
    city: maybe(string),
    latitude: number,
    longitude: number,
    mullvad_exit_ip: boolean,
    hostname: maybe(string),
    bridge_hostname: maybe(string),
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

const locationConstraintSchema = constraint(
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
);

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
      ipv4_gateway: string,
      ipv6_gateway: maybe(string),
    }),
  }),
);

const relaySettingsSchema = oneOf(
  object({
    normal: partialObject({
      location: locationConstraintSchema,
      tunnel_protocol: constraint(enumeration('wireguard', 'openvpn')),
      wireguard_constraints: partialObject({
        port: constraint(number),
      }),
      openvpn_constraints: partialObject({
        port: constraint(number),
        protocol: constraint(enumeration('udp', 'tcp')),
      }),
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
              active: boolean,
              weight: number,
              bridges: maybe(
                partialObject({
                  shadowsocks: arrayOf(
                    object({
                      port: number,
                      cipher: string,
                      password: string,
                      protocol: enumeration('tcp', 'udp'),
                    }),
                  ),
                }),
              ),
              tunnels: maybe(
                partialObject({
                  openvpn: arrayOf(
                    partialObject({
                      port: number,
                      protocol: string,
                    }),
                  ),
                  wireguard: arrayOf(
                    partialObject({
                      port_ranges: arrayOf(arrayOf(number)),
                      public_key: string,
                    }),
                  ),
                }),
              ),
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

const bridgeSettingsSchema = oneOf(
  partialObject({ normal: partialObject({ location: locationConstraintSchema }) }),
  partialObject({ custom: openVpnProxySchema }),
);

const tunnelOptionsSchema = partialObject({
  openvpn: partialObject({
    mssfix: maybe(number),
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

const voucherResponseSchema = partialObject({
  new_expiry: string,
});

const tunnelStateSchema = oneOf(
  object({
    state: enumeration('disconnecting'),
    details: enumeration('nothing', 'block', 'reconnect'),
  }),
  object({
    state: enumeration('connecting', 'connected'),
    details: object({
      endpoint: partialObject({
        address: string,
        protocol: enumeration('tcp', 'udp'),
        tunnel_type: enumeration('wireguard', 'openvpn'),
        proxy: maybe(
          partialObject({
            address: string,
            protocol: enumeration('tcp', 'udp'),
            proxy_type: enumeration('shadowsocks', 'custom'),
          }),
        ),
      }),
      location: maybe(locationSchema),
    }),
  }),
  object({
    state: enumeration('error'),
    details: object({
      block_failure: maybe(
        object({
          reason: enumeration('generic', 'locked'),
          details: maybe(
            object({
              name: string,
              pid: number,
            }),
          ),
        }),
      ),
      cause: oneOf(
        object({
          reason: enumeration(
            'ipv6_unavailable',
            'set_dns_error',
            'start_tunnel_error',
            'is_offline',
            'tap_adapter_problem',
          ),
        }),
        object({
          reason: enumeration('set_firewall_policy_error'),
          details: object({
            reason: enumeration('generic', 'locked'),
            details: maybe(
              object({
                name: string,
                pid: number,
              }),
            ),
          }),
        }),
        object({
          reason: enumeration('auth_failed'),
          details: maybe(string),
        }),
        object({
          reason: enumeration('tunnel_parameter_error'),
          details: enumeration(
            'no_matching_relay',
            'no_matching_bridge_relay',
            'no_wireguard_key',
            'custom_tunnel_host_resultion_error',
          ),
        }),
      ),
    }),
  }),
  object({
    state: enumeration('connected', 'connecting', 'disconnected'),
  }),
);

const appVersionInfoSchema = partialObject({
  supported: boolean,
  suggested_upgrade: maybe(string),
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
  // Only meant to be used by DaemonRpc
  // @internal
  public subscriptionId?: string | number;

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
  show_beta_releases: boolean,
  bridge_settings: bridgeSettingsSchema,
  bridge_state: enumeration('on', 'auto', 'off'),
  relay_settings: relaySettingsSchema,
  tunnel_options: tunnelOptionsSchema,
});

const wireguardPublicKey = object({
  key: string,
  created: string,
});

const keygenEventSchema = oneOf(
  enumeration('too_many_keys', 'generation_failure'),
  object({
    new_key: object({
      key: string,
      created: string,
    }),
  }),
);

const daemonEventSchema = oneOf(
  object({
    tunnel_state: tunnelStateSchema,
  }),
  object({
    settings: settingsSchema,
  }),
  object({
    relay_list: relayListSchema,
  }),
  object({
    wireguard_key: keygenEventSchema,
  }),
  object({
    app_version_info: appVersionInfoSchema,
  }),
);

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

  public connect(connectionParams: { path: string }): Promise<void> {
    return this.transport.connect(connectionParams);
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

  public async getWwwAuthToken(): Promise<string> {
    const response = await this.transport.send('get_www_auth_token');
    try {
      return validate(string, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_www_auth_token', error);
    }
  }

  public async submitVoucher(voucherCode: string): Promise<VoucherResponse> {
    try {
      const response = await this.transport.send('submit_voucher', voucherCode);
      const new_expiry = validate(voucherResponseSchema, response).new_expiry;
      return { type: 'success', new_expiry };
    } catch (error) {
      if (error instanceof JsonRpcRemoteError) {
        switch (error.code) {
          case VoucherErrorCode.Invalid:
            return { type: 'invalid' };
          case VoucherErrorCode.AlreadyUsed:
            return { type: 'already_used' };
        }
      }
    }

    return { type: 'error' };
  }

  public async getRelayLocations(): Promise<IRelayList> {
    const response = await this.transport.send('get_relay_locations');
    try {
      return snakeCaseToCamelCase(validate(relayListSchema, response));
    } catch (error) {
      throw new ResponseParseError(`Invalid response from get_relay_locations: ${error}`, error);
    }
  }

  public async createNewAccount(): Promise<string> {
    const response = await this.transport.send('create_new_account');
    return validate(string, response);
  }

  public async setAccount(accountToken?: AccountToken): Promise<void> {
    await this.transport.send('set_account', [accountToken]);
  }

  public async updateRelaySettings(relaySettings: RelaySettingsUpdate): Promise<void> {
    await this.transport.send('update_relay_settings', [camelCaseToSnakeCase(relaySettings)]);
  }

  public async setAllowLan(allowLan: boolean): Promise<void> {
    await this.transport.send('set_allow_lan', [allowLan]);
  }

  public async setShowBetaReleases(showBetaReleases: boolean): Promise<void> {
    await this.transport.send('set_show_beta_releases', [showBetaReleases]);
  }

  public async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this.transport.send('set_enable_ipv6', [enableIpv6]);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this.transport.send('set_block_when_disconnected', [blockWhenDisconnected]);
  }

  public async setBridgeState(bridgeState: BridgeState): Promise<void> {
    await this.transport.send('set_bridge_state', [bridgeState]);
  }

  public async setBridgeSettings(bridgeSettings: BridgeSettings): Promise<void> {
    await this.transport.send('set_bridge_settings', [bridgeSettings]);
  }

  public async setOpenVpnMssfix(mssfix?: number): Promise<void> {
    await this.transport.send('set_openvpn_mssfix', [mssfix]);
  }

  public async setWireguardMtu(mtu?: number): Promise<void> {
    await this.transport.send('set_wireguard_mtu', [mtu]);
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

  public async reconnectTunnel(): Promise<void> {
    await this.transport.send('reconnect');
  }

  public async getLocation(): Promise<ILocation | undefined> {
    const response = await this.transport.send('get_current_location', [], NETWORK_CALL_TIMEOUT);
    try {
      const validatedObject = validate(locationSchema, response);
      if (validatedObject) {
        return snakeCaseToCamelCase(validatedObject);
      } else {
        return undefined;
      }
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_location', error);
    }
  }

  public async getState(): Promise<TunnelState> {
    const response = await this.transport.send('get_state');
    try {
      return snakeCaseToCamelCase(validate(tunnelStateSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_state', error);
    }
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.transport.send('get_settings');
    try {
      return snakeCaseToCamelCase(validate(settingsSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_settings', error);
    }
  }

  public async subscribeDaemonEventListener(
    listener: SubscriptionListener<DaemonEvent>,
  ): Promise<void> {
    const subscriptionId = await this.transport.subscribe('daemon_event', (payload) => {
      let daemonEvent: DaemonEvent;

      try {
        daemonEvent = snakeCaseToCamelCase(validate(daemonEventSchema, payload));
      } catch (error) {
        listener.onError(new ResponseParseError('Invalid payload from daemon_event', error));
        return;
      }

      listener.onEvent(daemonEvent);
    });

    listener.subscriptionId = subscriptionId;
  }

  public async unsubscribeDaemonEventListener(
    listener: SubscriptionListener<DaemonEvent>,
  ): Promise<void> {
    if (listener.subscriptionId) {
      return this.transport.unsubscribe('daemon_event', listener.subscriptionId);
    }
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

  public async generateWireguardKey(): Promise<KeygenEvent> {
    const response = await this.transport.send('generate_wireguard_key');
    try {
      const validatedResponse = validate(keygenEventSchema, response);
      switch (validatedResponse) {
        case 'too_many_keys':
        case 'generation_failure':
          return validatedResponse;
        default:
          return snakeCaseToCamelCase(validatedResponse as object);
      }
    } catch (error) {
      throw new ResponseParseError(`Invalid response from generate_wireguard_key ${error}`);
    }
  }

  public async getWireguardKey(): Promise<IWireguardPublicKey | undefined> {
    const response = await this.transport.send('get_wireguard_key');
    try {
      return validate(maybe(wireguardPublicKey), response) || undefined;
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_wireguard_key');
    }
  }

  public async verifyWireguardKey(): Promise<boolean> {
    const response = await this.transport.send('verify_wireguard_key');
    try {
      return validate(boolean, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from verify_wireguard_key');
    }
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    const response = await this.transport.send('get_version_info', [], NETWORK_CALL_TIMEOUT);
    try {
      return snakeCaseToCamelCase(validate(appVersionInfoSchema, response));
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_version_info');
    }
  }
}
