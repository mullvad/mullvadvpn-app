export interface IAccountData {
  expiry: string;
}
export type AccountToken = string;
export type Ip = string;
export interface ILocation {
  ip?: string;
  country: string;
  city?: string;
  latitude: number;
  longitude: number;
  mullvadExitIp: boolean;
  hostname?: string;
}

export type BlockReason =
  | {
      reason:
        | 'ipv6_unavailable'
        | 'set_firewall_policy_error'
        | 'set_dns_error'
        | 'start_tunnel_error'
        | 'no_matching_relay'
        | 'is_offline'
        | 'tap_adapter_problem';
    }
  | { reason: 'auth_failed'; details?: string };

export type AfterDisconnect = 'nothing' | 'block' | 'reconnect';

export type TunnelState = 'connecting' | 'connected' | 'disconnecting' | 'disconnected' | 'blocked';

export type TunnelType = 'wireguard' | 'openvpn';

export type RelayProtocol = 'tcp' | 'udp';

export interface ITunnelEndpoint {
  address: string;
  protocol: RelayProtocol;
  tunnel: TunnelType;
}

export type TunnelStateTransition =
  | { state: 'disconnected' }
  | { state: 'connecting'; details?: ITunnelEndpoint }
  | { state: 'connected'; details: ITunnelEndpoint }
  | { state: 'disconnecting'; details: AfterDisconnect }
  | { state: 'blocked'; details: BlockReason };

export type RelayLocation =
  | { hostname: [string, string, string] }
  | { city: [string, string] }
  | { country: string };

export interface IOpenVpnConstraints {
  port: 'any' | { only: number };
  protocol: 'any' | { only: RelayProtocol };
}

export interface IWireguardConstraints {
  port: 'any' | { only: number };
}

type TunnelConstraints<OpenVpn, Wireguard> = { wireguard: Wireguard } | { openvpn: OpenVpn };

interface IRelaySettingsNormal<TTunnelConstraints> {
  location:
    | 'any'
    | {
        only: RelayLocation;
      };
  tunnel:
    | 'any'
    | {
        only: TTunnelConstraints;
      };
}

export type ConnectionConfig =
  | {
      openvpn: {
        endpoint: {
          ip: string;
          port: number;
          protocol: RelayProtocol;
        };
        username: string;
      };
    }
  | {
      wireguard: {
        tunnel: {
          private_key: string;
          addresses: string[];
        };
        peer: {
          public_key: string;
          addresses: string[];
          endpoint: string;
        };
        gateway: string;
      };
    };

// types describing the structure of RelaySettings
export interface IRelaySettingsCustom {
  host: string;
  config: ConnectionConfig;
}
export type RelaySettings =
  | {
      normal: IRelaySettingsNormal<TunnelConstraints<IOpenVpnConstraints, IWireguardConstraints>>;
    }
  | {
      customTunnelEndpoint: IRelaySettingsCustom;
    };

// types describing the partial update of RelaySettings
export type RelaySettingsNormalUpdate = Partial<
  IRelaySettingsNormal<
    TunnelConstraints<Partial<IOpenVpnConstraints>, Partial<IWireguardConstraints>>
  >
>;

export type RelaySettingsUpdate =
  | {
      normal: RelaySettingsNormalUpdate;
    }
  | {
      customTunnelEndpoint: IRelaySettingsCustom;
    };

export interface IRelayList {
  countries: IRelayListCountry[];
}

export interface IRelayListCountry {
  name: string;
  code: string;
  cities: IRelayListCity[];
}

export interface IRelayListCity {
  name: string;
  code: string;
  latitude: number;
  longitude: number;
  relays: IRelayListHostname[];
}

export interface IRelayListHostname {
  hostname: string;
  ipv4AddrIn: string;
  includeInCountry: boolean;
  weight: number;
}

export interface ITunnelOptions {
  openvpn: {
    mssfix?: number;
    proxy?: ProxySettings;
  };
  wireguard: {
    mtu?: number;
    // Only relevant on Linux
    fwmark?: number;
  };
  generic: {
    enableIpv6: boolean;
  };
}

export type ProxySettings = ILocalProxySettings | IRemoteProxySettings | IShadowsocksProxySettings;

export interface ILocalProxySettings {
  port: number;
  peer: string;
}

export interface IRemoteProxySettings {
  address: string;
  auth?: IRemoteProxyAuth;
}

export interface IRemoteProxyAuth {
  username: string;
  password: string;
}

export interface IShadowsocksProxySettings {
  peer: string;
  password: string;
  cipher: string;
}

export interface IAppVersionInfo {
  currentIsSupported: boolean;
  latest: {
    latestStable: string;
    latest: string;
  };
}

export interface ISettings {
  accountToken?: AccountToken;
  allowLan: boolean;
  autoConnect: boolean;
  blockWhenDisconnected: boolean;
  relaySettings: RelaySettings;
  tunnelOptions: ITunnelOptions;
}

export interface ISocketAddress {
  host: string;
  port: number;
}

export function parseSocketAddress(socketAddrStr: string): ISocketAddress {
  const re = new RegExp(/(.+):(\d+)$/);
  const matches = socketAddrStr.match(re);

  if (!matches || matches.length < 3) {
    throw new Error(`Failed to parse socket address from address string '${socketAddrStr}'`);
  }
  const socketAddress: ISocketAddress = {
    host: matches[1],
    port: Number(matches[2]),
  };
  return socketAddress;
}

export function compareRelayLocation(lhs: RelayLocation, rhs: RelayLocation) {
  if ('country' in lhs && 'country' in rhs && lhs.country && rhs.country) {
    return lhs.country === rhs.country;
  } else if ('city' in lhs && 'city' in rhs && lhs.city && rhs.city) {
    return lhs.city[0] === rhs.city[0] && lhs.city[1] === rhs.city[1];
  } else if ('hostname' in lhs && 'hostname' in rhs && lhs.hostname && rhs.hostname) {
    return (
      lhs.hostname[0] === rhs.hostname[0] &&
      lhs.hostname[1] === rhs.hostname[1] &&
      lhs.hostname[2] === rhs.hostname[2]
    );
  } else {
    return false;
  }
}

export function compareRelayLocationLoose(lhs?: RelayLocation, rhs?: RelayLocation) {
  if (lhs && rhs) {
    return compareRelayLocation(lhs, rhs);
  } else {
    return lhs === rhs;
  }
}
