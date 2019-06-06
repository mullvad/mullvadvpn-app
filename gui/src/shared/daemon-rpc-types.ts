export interface IAccountData {
  expiry: string;
}
export type AccountToken = string;
export type Ip = string;
export interface ILocation {
  ipv4?: string;
  ipv6?: string;
  country: string;
  city?: string;
  latitude: number;
  longitude: number;
  mullvadExitIp: boolean;
  hostname?: string;
  bridgeHostname?: string;
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
export function tunnelTypeToString(tunnel: TunnelType): string {
  switch (tunnel) {
    case 'wireguard':
      return 'WireGuard';
    case 'openvpn':
      return 'OpenVPN';
    default:
      return '';
  }
}

export type RelayProtocol = 'tcp' | 'udp';

export type ProxyType = 'shadowsocks' | 'custom';
export function proxyTypeToString(proxy: ProxyType): string {
  switch (proxy) {
    case 'shadowsocks':
      return 'Shadowsocks';
    case 'custom':
      return 'Custom';
    default:
      return '';
  }
}

export interface ITunnelEndpoint {
  address: string;
  protocol: RelayProtocol;
  tunnelType: TunnelType;
  proxy?: IProxyEndpoint;
}

export interface IProxyEndpoint {
  address: string;
  protocol: RelayProtocol;
  proxyType: ProxyType;
}

export type DaemonEvent =
  | { stateTransition: TunnelStateTransition }
  | { settings: ISettings }
  | { relayList: IRelayList };

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
          privateKey: string;
          addresses: string[];
        };
        peer: {
          publicKey: string;
          addresses: string[];
          endpoint: string;
        };
        ipv4Gateway: string;
        ipv6Gateway?: string;
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
  tunnels?: IRelayTunnels;
  bridges?: IRelayBridges;
}

export interface IRelayTunnels {
  openvpn: IOpenVpnTunnelData[];
  wireguard: IWireguardTunnelData[];
}

export interface IRelayBridges {
  shadowsocks: IShadowsocksEndpointData[];
}

export interface IOpenVpnTunnelData {
  port: number;
  protocol: RelayProtocol;
}

export interface IWireguardTunnelData {
  // Port ranges are an array of pairs, such as [[53,53], [10_000, 60_000]],
  // which in this case translates that the specific tunnel can be connected on
  // port 53 and ports 10'000 through 60'000.
  portRanges: Array<[number, number]>;
  // Public key of the tunnel.
  publicKey: string;
}

export interface IShadowsocksEndpointData {
  port: number;
  cipher: string;
  password: string;
  protocol: RelayProtocol;
}

export interface ITunnelOptions {
  openvpn: {
    mssfix?: number;
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
  latestStable: string;
  latest: string;
}

export interface ISettings {
  accountToken?: AccountToken;
  allowLan: boolean;
  autoConnect: boolean;
  blockWhenDisconnected: boolean;
  relaySettings: RelaySettings;
  tunnelOptions: ITunnelOptions;
  bridgeSettings: BridgeSettings;
  bridgeState: BridgeState;
}

export type BridgeState = 'auto' | 'on' | 'off';

export interface IBridgeConstraints {
  location:
    | 'any'
    | {
        only: RelayLocation;
      };
}

export type BridgeSettings = ProxySettings | IBridgeConstraints;

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
