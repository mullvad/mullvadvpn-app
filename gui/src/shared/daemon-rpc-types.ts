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

export type FirewallPolicyError =
  | { reason: 'generic' }
  | {
      reason: 'locked';
      details?: {
        name: string;
        pid: number;
      };
    };

export type TunnelParameterError =
  | 'no_matching_relay'
  | 'no_matching_bridge_relay'
  | 'no_wireguard_key'
  | 'custom_tunnel_host_resultion_error';

export type ErrorStateCause =
  | {
      reason:
        | 'ipv6_unavailable'
        | 'set_dns_error'
        | 'start_tunnel_error'
        | 'is_offline'
        | 'tap_adapter_problem';
    }
  | { reason: 'set_firewall_policy_error'; details: FirewallPolicyError }
  | { reason: 'tunnel_parameter_error'; details: TunnelParameterError }
  | { reason: 'auth_failed'; details?: string };

export type AfterDisconnect = 'nothing' | 'block' | 'reconnect';

export type TunnelType = 'any' | 'wireguard' | 'openvpn';
export function tunnelTypeToString(tunnel: TunnelType): string {
  switch (tunnel) {
    case 'wireguard':
      return 'WireGuard';
    case 'openvpn':
      return 'OpenVPN';
    case 'any':
      return '';
  }
}

export type RelayProtocol = 'any' | 'tcp' | 'udp';

export type Constraint<T> = 'any' | { only: T };
export type LiftedConstraint<T> = 'any' | T;

export function liftConstraint<T>(constraint: Constraint<T>): LiftedConstraint<T> {
  return constraint === 'any' ? constraint : constraint.only;
}

export type ProxyType = 'shadowsocks' | 'custom';
export function proxyTypeToString(proxy: ProxyType): string {
  switch (proxy) {
    case 'shadowsocks':
      return 'Shadowsocks';
    case 'custom':
      return 'custom bridge';
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
  | { tunnelState: TunnelState }
  | { settings: ISettings }
  | { relayList: IRelayList }
  | { wireguardKey: KeygenEvent }
  | { appVersionInfo: IAppVersionInfo };

export interface ITunnelStateRelayInfo {
  endpoint: ITunnelEndpoint;
  location?: ILocation;
}

export type TunnelState =
  | { state: 'disconnected' }
  | { state: 'connecting'; details?: ITunnelStateRelayInfo }
  | { state: 'connected'; details: ITunnelStateRelayInfo }
  | { state: 'disconnecting'; details: AfterDisconnect }
  | { state: 'error'; details: IErrorState };

export interface IErrorState {
  blockFailure?: FirewallPolicyError;
  cause: ErrorStateCause;
}

export type RelayLocation =
  | { hostname: [string, string, string] }
  | { city: [string, string] }
  | { country: string };

export interface IOpenVpnConstraints {
  port: Constraint<number>;
  protocol: Constraint<RelayProtocol>;
}

export interface IWireguardConstraints {
  port: Constraint<number>;
}

export type TunnelProtocol = 'wireguard' | 'openvpn';

interface IRelaySettingsNormal<OpenVpn, Wireguard> {
  location: Constraint<RelayLocation>;
  tunnelProtocol: Constraint<TunnelProtocol>;
  openvpnConstraints: OpenVpn;
  wireguardConstraints: Wireguard;
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
      normal: IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints>;
    }
  | {
      customTunnelEndpoint: IRelaySettingsCustom;
    };

// types describing the partial update of RelaySettings
export type RelaySettingsNormalUpdate = Partial<
  IRelaySettingsNormal<Partial<IOpenVpnConstraints>, Partial<IWireguardConstraints>>
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
  active: boolean;
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
  portRanges: Array<IPortRange>;
  // Public key of the tunnel.
  publicKey: string;
}

export interface IPortRange {
  first: number;
  last: number;
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
  supported: boolean;
  suggestedUpgrade?: string;
}

export interface ISettings {
  accountToken?: AccountToken;
  allowLan: boolean;
  autoConnect: boolean;
  blockWhenDisconnected: boolean;
  showBetaReleases: boolean;
  relaySettings: RelaySettings;
  tunnelOptions: ITunnelOptions;
  bridgeSettings: BridgeSettings;
  bridgeState: BridgeState;
}

export type KeygenEvent = INewWireguardKey | KeygenFailure;
export type KeygenFailure = 'too_many_keys' | 'generation_failure';

export interface INewWireguardKey {
  newKey: IWireguardPublicKey;
}

export interface IWireguardPublicKey {
  key: string;
  created: string;
}

export type BridgeState = 'auto' | 'on' | 'off';

export interface IBridgeConstraints {
  location: Constraint<RelayLocation>;
}

export type BridgeSettings = { normal: IBridgeConstraints } | { custom: ProxySettings };

export interface ISocketAddress {
  host: string;
  port: number;
}

export type VoucherResponse =
  | { type: 'success'; new_expiry: string }
  | { type: 'invalid' | 'already_used' | 'error' };

export enum VoucherErrorCode {
  Invalid = -400,
  AlreadyUsed = -401,
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

export function relayLocationComponents(location: RelayLocation): string[] {
  if ('country' in location) {
    return [location.country];
  } else if ('city' in location) {
    return location.city;
  } else {
    return location.hostname;
  }
}

export function compareRelayLocation(lhs: RelayLocation, rhs: RelayLocation): boolean {
  const lhsComponents = relayLocationComponents(lhs);
  const rhsComponents = relayLocationComponents(rhs);

  if (lhsComponents.length === rhsComponents.length) {
    return lhsComponents.every((value, index) => value === rhsComponents[index]);
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
