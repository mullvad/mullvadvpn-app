// @flow

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

export type AppVersionInfo = {
  currentIsSupported: boolean,
  latest: {
    latestStable: string,
    latest: string,
  },
};

export type Settings = {
  accountToken: ?AccountToken,
  allowLan: boolean,
  autoConnect: boolean,
  blockWhenDisconnected: boolean,
  relaySettings: RelaySettings,
  tunnelOptions: TunnelOptions,
};
