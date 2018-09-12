// @flow

import JsonRpcClient, {
  RemoteError as JsonRpcRemoteError,
  TimeOutError as JsonRpcTimeOutError,
  SocketTransport,
} from './jsonrpc-client';
import { CommunicationError, InvalidAccountError, NoDaemonError } from '../errors';

import {
  object,
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
  ip: Ip,
  country: string,
  city: ?string,
  latitude: number,
  longitude: number,
  mullvad_exit_ip: boolean,
};
const LocationSchema = object({
  ip: string,
  country: string,
  city: maybe(string),
  latitude: number,
  longitude: number,
  mullvad_exit_ip: boolean,
});

export type BlockReason =
  | 'ipv6_unavailable'
  | 'set_security_policy_error'
  | 'start_tunnel_error'
  | 'no_matching_relay';

export type TunnelState = 'connecting' | 'connected' | 'disconnecting' | 'disconnected' | 'blocked';

export type TunnelStateTransition =
  | { state: 'disconnecting' | 'disconnected' | 'connecting' | 'connected' }
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
  tunnel: {
    openvpn: {
      port: number,
      protocol: RelayProtocol,
    },
  },
};
export type RelaySettings =
  | {|
      normal: RelaySettingsNormal<TunnelConstraints<OpenVpnConstraints>>,
    |}
  | {|
      custom_tunnel_endpoint: RelaySettingsCustom,
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
      custom_tunnel_endpoint: RelaySettingsCustom,
    |};

const constraint = <T>(constraintValue: SchemaNode<T>) => {
  return oneOf(
    string, // any
    object({
      only: constraintValue,
    }),
  );
};

const RelaySettingsSchema = oneOf(
  object({
    normal: object({
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
        object({
          openvpn: object({
            port: constraint(number),
            protocol: constraint(enumeration('udp', 'tcp')),
          }),
        }),
      ),
    }),
  }),
  object({
    custom_tunnel_endpoint: object({
      host: string,
      tunnel: object({
        openvpn: object({
          port: number,
          protocol: enumeration('udp', 'tcp'),
        }),
      }),
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
  ipv4_addr_in: string,
  ipv4_addr_exit: string,
  include_in_country: boolean,
  weight: number,
};

const RelayListSchema = object({
  countries: arrayOf(
    object({
      name: string,
      code: string,
      cities: arrayOf(
        object({
          name: string,
          code: string,
          latitude: number,
          longitude: number,
          relays: arrayOf(
            object({
              hostname: string,
              ipv4_addr_in: string,
              ipv4_addr_exit: string,
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
};

const TunnelOptionsSchema = object({
  enable_ipv6: boolean,
  openvpn: object({
    mssfix: maybe(number),
  }),
});

const AccountDataSchema = object({
  expiry: string,
});

const allBlockReasons: Array<BlockReason> = [
  'ipv6_unavailable',
  'set_security_policy_error',
  'start_tunnel_error',
  'no_matching_relay',
];
const TunnelStateTransitionSchema = oneOf(
  object({
    state: enumeration('blocked'),
    details: enumeration(...allBlockReasons),
  }),
  object({
    state: enumeration('connected', 'connecting', 'disconnected', 'disconnecting'),
  }),
);

export type AppVersionInfo = {
  currentIsSupported: boolean,
  latest: {
    latestStable: string,
    latest: string,
  },
};

const AppVersionInfoSchema = object({
  current_is_supported: boolean,
  latest: object({
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

export interface DaemonRpcProtocol {
  connect({ path: string }): void;
  disconnect(): void;
  getAccountData(AccountToken): Promise<AccountData>;
  getRelayLocations(): Promise<RelayList>;
  getAccount(): Promise<?AccountToken>;
  setAccount(accountToken: ?AccountToken): Promise<void>;
  updateRelaySettings(RelaySettingsUpdate): Promise<void>;
  getRelaySettings(): Promise<RelaySettings>;
  setAllowLan(boolean): Promise<void>;
  getAllowLan(): Promise<boolean>;
  setEnableIpv6(boolean): Promise<void>;
  getTunnelOptions(): Promise<TunnelOptions>;
  setAutoConnect(boolean): Promise<void>;
  getAutoConnect(): Promise<boolean>;
  connectTunnel(): Promise<void>;
  disconnectTunnel(): Promise<void>;
  getLocation(): Promise<Location>;
  getState(): Promise<TunnelStateTransition>;
  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void>;
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
      return validate(RelayListSchema, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_relay_locations', error);
    }
  }

  async getAccount(): Promise<?AccountToken> {
    const response = await this._transport.send('get_account');
    if (response === null || typeof response === 'string') {
      return response;
    } else {
      throw new ResponseParseError('Invalid response from get_account', null);
    }
  }

  async setAccount(accountToken: ?AccountToken): Promise<void> {
    await this._transport.send('set_account', accountToken);
  }

  async updateRelaySettings(relaySettings: RelaySettingsUpdate): Promise<void> {
    await this._transport.send('update_relay_settings', [relaySettings]);
  }

  async getRelaySettings(): Promise<RelaySettings> {
    const response = await this._transport.send('get_relay_settings');
    try {
      const validatedObject = validate(RelaySettingsSchema, response);

      /* $FlowFixMe:
        There is no way to express constraints with string literals, i.e:

        RelaySettingsSchema constraint:
          oneOf(string, object)

        RelaySettings constraint:
          'any' | object

        These two are incompatible so we simply enforce the type for now.
      */
      return ((validatedObject: any): RelaySettings);
    } catch (e) {
      throw new ResponseParseError('Invalid response from get_relay_settings', e);
    }
  }

  async setAllowLan(allowLan: boolean): Promise<void> {
    await this._transport.send('set_allow_lan', [allowLan]);
  }

  async getAllowLan(): Promise<boolean> {
    const response = await this._transport.send('get_allow_lan');
    if (typeof response === 'boolean') {
      return response;
    } else {
      throw new ResponseParseError('Invalid response from get_allow_lan', null);
    }
  }

  async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this._transport.send('set_enable_ipv6', [enableIpv6]);
  }

  async getTunnelOptions(): Promise<TunnelOptions> {
    const response = await this._transport.send('get_tunnel_options');
    try {
      const validatedObject = validate(TunnelOptionsSchema, response);

      return {
        enableIpv6: validatedObject.enable_ipv6,
      };
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_tunnel_options', error);
    }
  }

  async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this._transport.send('set_auto_connect', [autoConnect]);
  }

  async getAutoConnect(): Promise<boolean> {
    const response = await this._transport.send('get_auto_connect');
    if (typeof response === 'boolean') {
      return response;
    } else {
      throw new ResponseParseError('Invalid response from get_auto_connect', null);
    }
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
      return validate(LocationSchema, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_current_location', error);
    }
  }

  async getState(): Promise<TunnelStateTransition> {
    const response = await this._transport.send('get_state');
    try {
      return validate(TunnelStateTransitionSchema, response);
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_state', error);
    }
  }

  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void> {
    return this._transport.subscribe('new_state', (payload) => {
      try {
        const newState = validate(TunnelStateTransitionSchema, payload);
        listener._onEvent(newState);
      } catch (error) {
        listener._onError(new ResponseParseError('Invalid payload from new_state', error));
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
      const versionInfo = validate(AppVersionInfoSchema, response);
      return {
        currentIsSupported: versionInfo.current_is_supported,
        latest: {
          latestStable: versionInfo.latest.latest_stable,
          latest: versionInfo.latest.latest,
        },
      };
    } catch (error) {
      throw new ResponseParseError('Invalid response from get_version_info', null);
    }
  }
}
