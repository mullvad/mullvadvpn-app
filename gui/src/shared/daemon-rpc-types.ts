export interface IAccountData {
  expiry: string;
}

export type AccountDataError = {
  type: 'error';
  error: 'invalid-account' | 'too-many-devices' | 'list-devices' | 'communication';
};

export type AccountDataResponse = ({ type: 'success' } & IAccountData) | AccountDataError;

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
  entryHostname?: string;
  provider?: string;
}

export enum FirewallPolicyErrorType {
  generic,
  locked,
}

export type FirewallPolicyError =
  | { type: FirewallPolicyErrorType.generic }
  | {
      type: FirewallPolicyErrorType.locked;
      name: string;
      pid: number;
    };

export enum ErrorStateCause {
  authFailed,
  ipv6Unavailable,
  setFirewallPolicyError,
  setDnsError,
  startTunnelError,
  tunnelParameterError,
  isOffline,
  splitTunnelError,
}

export enum AuthFailedError {
  unknown,
  invalidAccount,
  expiredAccount,
  tooManyConnections,
}

export enum TunnelParameterError {
  noMatchingRelay,
  noMatchingBridgeRelay,
  noWireguardKey,
  customTunnelHostResolutionError,
}

export type ErrorState =
  | {
      cause:
        | ErrorStateCause.ipv6Unavailable
        | ErrorStateCause.setDnsError
        | ErrorStateCause.startTunnelError
        | ErrorStateCause.isOffline
        | ErrorStateCause.splitTunnelError;
      blockingError?: FirewallPolicyError;
    }
  | {
      cause: ErrorStateCause.authFailed;
      blockingError?: FirewallPolicyError;
      authFailedError: AuthFailedError;
    }
  | {
      cause: ErrorStateCause.tunnelParameterError;
      blockingError?: FirewallPolicyError;
      parameterError: TunnelParameterError;
    }
  | {
      cause: ErrorStateCause.setFirewallPolicyError;
      blockingError?: FirewallPolicyError;
      policyError: FirewallPolicyError;
    };

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

export type RelayProtocol = 'tcp' | 'udp';
export type EndpointObfuscationType = 'udp2tcp';

export type Constraint<T> = 'any' | { only: T };
export type LiftedConstraint<T> = 'any' | T;

export function liftConstraint<T>(constraint: Constraint<T>): LiftedConstraint<T> {
  return constraint === 'any' ? constraint : constraint.only;
}
export function wrapConstraint<T>(
  constraint: LiftedConstraint<T> | undefined | null,
): Constraint<T> {
  if (constraint) {
    return constraint === 'any' ? 'any' : { only: constraint };
  }
  return 'any';
}

export type ProxyType = 'shadowsocks' | 'custom';
export function proxyTypeToString(proxy: ProxyType): string {
  switch (proxy) {
    case 'shadowsocks':
      return 'Shadowsocks bridge';
    case 'custom':
      return 'custom bridge';
    default:
      return '';
  }
}

export enum Ownership {
  any,
  mullvadOwned,
  rented,
}

export interface ITunnelEndpoint {
  address: string;
  protocol: RelayProtocol;
  tunnelType: TunnelType;
  quantumResistant: boolean;
  proxy?: IProxyEndpoint;
  obfuscationEndpoint?: IObfuscationEndpoint;
  entryEndpoint?: IEndpoint;
}

export interface IEndpoint {
  address: string;
  transportProtocol: RelayProtocol;
}

export interface IObfuscationEndpoint {
  address: string;
  port: number;
  protocol: RelayProtocol;
  obfuscationType: EndpointObfuscationType;
}

export interface IProxyEndpoint {
  address: string;
  protocol: RelayProtocol;
  proxyType: ProxyType;
}

export type DaemonEvent =
  | { tunnelState: TunnelState }
  | { settings: ISettings }
  | { relayList: IRelayListWithEndpointData }
  | { appVersionInfo: IAppVersionInfo }
  | { device: DeviceEvent }
  | { deviceRemoval: Array<IDevice> };

export interface ITunnelStateRelayInfo {
  endpoint: ITunnelEndpoint;
  location?: ILocation;
}

export type TunnelState =
  | { state: 'disconnected' }
  | { state: 'connecting'; details?: ITunnelStateRelayInfo }
  | { state: 'connected'; details: ITunnelStateRelayInfo }
  | { state: 'disconnecting'; details: AfterDisconnect }
  | { state: 'error'; details: ErrorState };

export interface RelayLocationCountry extends Partial<RelayLocationCustomList> {
  country: string;
}

export interface RelayLocationCity extends RelayLocationCountry {
  city: string;
}

export interface RelayLocationRelay extends RelayLocationCity {
  hostname: string;
}

export interface RelayLocationCustomList {
  customList: string;
}

export type RelayLocationGeographical =
  | RelayLocationRelay
  | RelayLocationCountry
  | RelayLocationCity;

export type RelayLocation = RelayLocationGeographical | RelayLocationCustomList;

export interface IOpenVpnConstraints {
  port: Constraint<number>;
  protocol: Constraint<RelayProtocol>;
}

export interface IWireguardConstraints {
  port: Constraint<number>;
  ipVersion: Constraint<IpVersion>;
  useMultihop: boolean;
  entryLocation: Constraint<RelayLocation>;
}

export type TunnelProtocol = 'wireguard' | 'openvpn';

export type IpVersion = 'ipv4' | 'ipv6';

export interface IRelaySettingsNormal<OpenVpn, Wireguard> {
  location: Constraint<RelayLocation>;
  tunnelProtocol: Constraint<TunnelProtocol>;
  providers: string[];
  ownership: Ownership;
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

export interface IRelayListWithEndpointData {
  relayList: IRelayList;
  wireguardEndpointData: IWireguardEndpointData;
}

export interface IRelayList {
  countries: IRelayListCountry[];
}

export interface IWireguardEndpointData {
  portRanges: [number, number][];
  udp2tcpPorts: number[];
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
  provider: string;
  ipv4AddrIn: string;
  includeInCountry: boolean;
  active: boolean;
  weight: number;
  owned: boolean;
  endpointType: RelayEndpointType;
}

export type RelayEndpointType = 'wireguard' | 'openvpn' | 'bridge';

export interface ITunnelOptions {
  openvpn: {
    mssfix?: number;
  };
  wireguard: {
    mtu?: number;
    quantumResistant?: boolean;
  };
  generic: {
    enableIpv6: boolean;
  };
  dns: IDnsOptions;
}

export interface IDnsOptions {
  state: 'custom' | 'default';
  customOptions: {
    addresses: string[];
  };
  defaultOptions: {
    blockAds: boolean;
    blockTrackers: boolean;
    blockMalware: boolean;
    blockAdultContent: boolean;
    blockGambling: boolean;
    blockSocialMedia: boolean;
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
  suggestedIsBeta?: boolean;
}

export interface IAccountAndDevice {
  accountToken: AccountToken;
  device?: IDevice;
}

export type LoggedInDeviceState = { type: 'logged in'; accountAndDevice: IAccountAndDevice };
export type LoggedOutDeviceState = { type: 'logged out' | 'revoked' };

export type DeviceState = LoggedInDeviceState | LoggedOutDeviceState;

export type DeviceEvent =
  | { type: 'logged in' | 'updated' | 'rotated_key'; deviceState: LoggedInDeviceState }
  | { type: 'logged out' | 'revoked'; deviceState: LoggedOutDeviceState };

export interface IDevice {
  id: string;
  name: string;
  created: Date;
}

export interface IDeviceRemoval {
  accountToken: string;
  deviceId: string;
}

export type CustomLists = Array<ICustomList>;

export interface ICustomList {
  id: string;
  name: string;
  locations: Array<RelayLocationGeographical>;
}

export type CustomListError = { type: 'name already exists' };

export interface ISettings {
  allowLan: boolean;
  autoConnect: boolean;
  blockWhenDisconnected: boolean;
  showBetaReleases: boolean;
  relaySettings: RelaySettings;
  tunnelOptions: ITunnelOptions;
  bridgeSettings: BridgeSettings;
  bridgeState: BridgeState;
  splitTunnel: SplitTunnelSettings;
  obfuscationSettings: ObfuscationSettings;
  customLists: CustomLists;
}

export type BridgeState = 'auto' | 'on' | 'off';

export type SplitTunnelSettings = {
  enableExclusions: boolean;
  appsList: string[];
};

export type Udp2TcpObfuscationSettings = {
  port: Constraint<number>;
};

export enum ObfuscationType {
  auto,
  off,
  udp2tcp,
}

export type ObfuscationSettings = {
  selectedObfuscation: ObfuscationType;
  udp2tcpSettings: Udp2TcpObfuscationSettings;
};

export interface IBridgeConstraints {
  location: Constraint<RelayLocation>;
  providers: string[];
  ownership: Ownership;
}

export type BridgeSettings = { normal: IBridgeConstraints } | { custom: ProxySettings };

export interface ISocketAddress {
  host: string;
  port: number;
}

export type VoucherResponse =
  | { type: 'success'; newExpiry: string; secondsAdded: number }
  | { type: 'invalid' | 'already_used' | 'error' };

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

export function compareRelayLocationCount(lhs: RelayLocation, rhs: RelayLocation): boolean {
  if (
    ('count' in lhs || 'count' in rhs) &&
    !('count' in lhs && 'count' in rhs && lhs.count === rhs.count)
  ) {
    return false;
  }

  return compareRelayLocation(lhs, rhs);
}

export function compareRelayLocation(lhs: RelayLocation, rhs: RelayLocation): boolean {
  if (
    ('customList' in lhs || 'customList' in rhs) &&
    !('customList' in lhs && 'customList' in rhs && lhs.customList === rhs.customList)
  ) {
    return false;
  }

  return compareRelayLocationGeographical(lhs, rhs);
}

export function compareRelayLocationGeographical(lhs: RelayLocation, rhs: RelayLocation): boolean {
  if (
    ('country' in lhs || 'country' in rhs) &&
    !('country' in lhs && 'country' in rhs && lhs.country === rhs.country)
  ) {
    return false;
  }

  if (
    ('city' in lhs || 'city' in rhs) &&
    !('city' in lhs && 'city' in rhs && lhs.city === rhs.city)
  ) {
    return false;
  }

  if (
    ('hostname' in lhs || 'hostname' in rhs) &&
    !('hostname' in lhs && 'hostname' in rhs && lhs.hostname === rhs.hostname)
  ) {
    return false;
  }

  return true;
}

export function compareRelayLocationLoose(lhs?: RelayLocation, rhs?: RelayLocation) {
  if (lhs && rhs) {
    return compareRelayLocation(lhs, rhs);
  } else {
    return lhs === rhs;
  }
}
