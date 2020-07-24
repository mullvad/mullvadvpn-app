import * as grpc from '@grpc/grpc-js';
import {
  BoolValue,
  StringValue,
  UInt32Value,
} from 'google-protobuf/google/protobuf/wrappers_pb.js';
import { Empty } from 'google-protobuf/google/protobuf/empty_pb.js';
import { promisify } from 'util';
import {
  AccountToken,
  IRelayListCountry,
  IRelayListCity,
  IRelayListHostname,
  IWireguardTunnelData,
  IShadowsocksEndpointData,
  RelayProtocol,
  BridgeState,
  ILocation,
  IAppVersionInfo,
  IAccountData,
  IOpenVpnTunnelData,
  TunnelState,
  AfterDisconnect,
  IErrorState,
  ErrorStateCause,
  TunnelParameterError,
  ITunnelStateRelayInfo,
  TunnelType,
  IProxyEndpoint,
  ProxyType,
  KeygenEvent,
  IWireguardPublicKey,
  ISettings,
  ConnectionConfig,
} from '../shared/daemon-rpc-types';
import * as managementInterface from './management_interface/management_interface_grpc_pb';
import {
  AccountData,
  BridgeState as GrpcBridgeState,
  TunnelState as GrpcTunnelState,
  AfterDisconnect as GrpcAfterDisconnect,
  TunnelType as GrpcTunnelType,
  ProxyType as GrpcProxyType,
  KeygenEvent as GrpcKeygenEvent,
  RelaySettings as GrpcRelaySettings,
  ConnectionConfig as GrpcConnectionConfig,
  VoucherSubmission,
  RelayListCountry,
  RelayListCity,
  Relay,
  WireguardEndpointData,
  ShadowsocksEndpointData,
  TransportProtocol,
  GeoIpLocation,
  AccountHistory,
  AppVersionInfo,
  OpenVpnEndpointData,
  ErrorState,
  TunnelStateRelayInfo,
  ProxyEndpoint,
  PublicKey,
  Settings,
} from './management_interface/management_interface_pb';

const NETWORK_CALL_TIMEOUT = 10000;

export interface ErrorResponse {
  code: number;
  details: string;
}

const ManagementServiceClient = grpc.makeClientConstructor(
  // @ts-ignore
  managementInterface['mullvad_daemon.management_interface.ManagementService'],
  'ManagementService',
);

const noConnectionError = Error('No connection established to daemon');

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

type CallFunctionArgument<T, R> =
  | ((arg: T, callback: (error: Error | null, result: R) => void) => void)
  | undefined;

export class GrpcClient {
  private client?: managementInterface.ManagementServiceClient;
  private connectionObservers: ConnectionObserver[] = [];

  private deadlineFromNow() {
    return Date.now() + NETWORK_CALL_TIMEOUT;
  }

  private callEmpty<R>(fn: CallFunctionArgument<Empty, R>): Promise<R> {
    return this.call<Empty, R>(fn, new Empty());
  }

  private callString<R>(fn: CallFunctionArgument<StringValue, R>, value?: string): Promise<R> {
    const googleString = new StringValue();

    if (value) {
      googleString.setValue(value);
    }

    return this.call<StringValue, R>(fn, googleString);
  }

  private callBool<R>(fn: CallFunctionArgument<BoolValue, R>, value?: boolean): Promise<R> {
    const googleBool = new BoolValue();

    if (value) {
      googleBool.setValue(value);
    }

    return this.call<BoolValue, R>(fn, googleBool);
  }

  private callNumber<R>(fn: CallFunctionArgument<UInt32Value, R>, value?: number): Promise<R> {
    const googleNumber = new UInt32Value();

    if (value) {
      googleNumber.setValue(value);
    }

    return this.call<UInt32Value, R>(fn, googleNumber);
  }

  private call<T, R>(fn: CallFunctionArgument<T, R>, arg: T): Promise<R> {
    if (this.client && fn) {
      return promisify<T, R>(fn.bind(this.client))(arg);
    } else {
      throw noConnectionError;
    }
  }

  public connect(connectionParams: { path: string }): Promise<void> {
    return new Promise((resolve, reject) => {
      const client = (new ManagementServiceClient(
        `unix://${connectionParams.path}`,
        grpc.credentials.createInsecure(),
      ) as unknown) as managementInterface.ManagementServiceClient;

      client.waitForReady(this.deadlineFromNow(), (error) => {
        if (error) {
          reject(error);
        } else {
          this.client = client;
          this.connectionObservers.forEach((connectionObserver) => connectionObserver.onOpen());
          resolve();
        }
      });
    });
  }

  public disconnect() {
    this.client?.close();
  }

  public addConnectionObserver(observer: ConnectionObserver) {
    this.connectionObservers.push(observer);
    observer.onOpen();
  }

  public removeConnectionObserver(observer: ConnectionObserver) {
    this.connectionObservers.splice(this.connectionObservers.indexOf(observer), 1);
  }

  public async getAccountData(accountToken: AccountToken): Promise<IAccountData> {
    const response = await this.callString<AccountData>(this.client?.getAccountData, accountToken);
    const expiry = response.getExpiry()!.toDate().toISOString();
    return { expiry };
  }

  public async getWwwAuthToken(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client?.getWwwAuthToken);
    return response.getValue();
  }

  public async submitVoucher(
    voucherCode: string,
  ): Promise<{ secondsAdded: number; newExpiry?: string }> {
    const response = await this.callString<VoucherSubmission>(
      this.client?.submitVoucher,
      voucherCode,
    );
    return {
      secondsAdded: response.getSecondsAdded(),
      newExpiry: response.getNewExpiry()?.toDate().toISOString(),
    };
  }

  public getRelayLocations(): Promise<IRelayListCountry[]> {
    if (this.client) {
      return new Promise((resolve, reject) => {
        const relayLocations: IRelayListCountry[] = [];
        const stream = this.client!.getRelayLocations(new Empty());
        stream.on('data', (country: RelayListCountry) =>
          relayLocations.push(convertRelayListCountry(country.toObject())),
        );
        stream.on('end', () => resolve(relayLocations));
        stream.on('close', reject);
      });
    } else {
      throw noConnectionError;
    }
  }

  public async createNewAccount(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client?.createNewAccount);
    return response.getValue();
  }

  public async setAccount(accountToken?: AccountToken): Promise<void> {
    await this.callString(this.client?.setAccount, accountToken);
  }

  public async setAllowLan(allowLan: boolean): Promise<void> {
    await this.callBool(this.client?.setAllowLan, allowLan);
  }

  public async setShowBetaReleases(showBetaReleases: boolean): Promise<void> {
    await this.callBool(this.client?.setShowBetaReleases, showBetaReleases);
  }

  public async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this.callBool(this.client?.setEnableIpv6, enableIpv6);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this.callBool(this.client?.setBlockWhenDisconnected, blockWhenDisconnected);
  }

  public async setBridgeState(bridgeState: BridgeState): Promise<void> {
    const bridgeStateMap = {
      auto: GrpcBridgeState.State.AUTO,
      on: GrpcBridgeState.State.ON,
      off: GrpcBridgeState.State.OFF,
    };

    const grpcBridgeState = new GrpcBridgeState();
    grpcBridgeState.setState(bridgeStateMap[bridgeState]);
    await this.call<GrpcBridgeState, Empty>(this.client?.setBridgeState, grpcBridgeState);
  }

  public async setOpenVpnMssfix(mssfix?: number): Promise<void> {
    await this.callNumber(this.client?.setOpenvpnMssfix, mssfix);
  }

  public async setWireguardMtu(mtu?: number): Promise<void> {
    await this.callNumber(this.client?.setWireguardMtu, mtu);
  }

  public async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this.callBool(this.client?.setAutoConnect, autoConnect);
  }

  public async connectTunnel(): Promise<void> {
    await this.callEmpty(this.client?.connectTunnel);
  }

  public async disconnectTunnel(): Promise<void> {
    await this.callEmpty(this.client?.disconnectTunnel);
  }

  public async reconnectTunnel(): Promise<void> {
    await this.callEmpty(this.client?.reconnectTunnel);
  }

  public async getLocation(): Promise<ILocation> {
    const response = await this.callEmpty<GeoIpLocation>(this.client?.getCurrentLocation);
    return response.toObject();
  }

  public async getState(): Promise<TunnelState> {
    const response = await this.callEmpty<GrpcTunnelState>(this.client?.getTunnelState);
    return convertTunnelState(response)!;
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.callEmpty<Settings>(this.client?.getSettings);
    return convertSettings(response)!;
  }

  public async getAccountHistory(): Promise<AccountToken[]> {
    const response = await this.callEmpty<AccountHistory>(this.client?.getAccountHistory);
    return response.toObject().tokenList;
  }

  public async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this.callString(this.client?.removeAccountFromHistory, accountToken);
  }

  public async getCurrentVersion(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client?.getCurrentVersion);
    return response.getValue();
  }

  public async generateWireguardKey(): Promise<KeygenEvent> {
    const response = await this.callEmpty<GrpcKeygenEvent>(this.client?.generateWireguardKey);
    switch (response.getEvent()) {
      case GrpcKeygenEvent.KeygenEvent.TOO_MANY_KEYS:
        return 'too_many_keys';
      case GrpcKeygenEvent.KeygenEvent.NEW_KEY: {
        const newKey = response.getNewKey();
        return newKey
          ? {
              newKey: {
                created: newKey.getCreated()!.toDate().toISOString(),
                key: convertWireguardKey(newKey.getKey()),
              },
            }
          : 'generation_failure';
      }
      case GrpcKeygenEvent.KeygenEvent.GENERATION_FAILURE:
        return 'generation_failure';
    }
  }

  public async getWireguardKey(): Promise<IWireguardPublicKey> {
    const response = await this.callEmpty<PublicKey>(this.client?.getWireguardKey);
    return {
      created: response.getCreated()!.toDate().toISOString(),
      key: convertWireguardKey(response.getKey()),
    };
  }

  public async verifyWireguardKey(): Promise<boolean> {
    const response = await this.callEmpty<BoolValue>(this.client?.verifyWireguardKey);
    return response.getValue();
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    const response = await this.callEmpty<AppVersionInfo>(this.client?.getVersionInfo);
    return response.toObject();
  }
}

function convertRelayListCountry(country: RelayListCountry.AsObject): IRelayListCountry {
  return {
    ...country,
    cities: country.citiesList.map(convertRelayListCity),
  };
}

function convertRelayListCity(city: RelayListCity.AsObject): IRelayListCity {
  return {
    ...city,
    relays: city.relaysList.map(convertRelayListRelay),
  };
}

function convertRelayListRelay(relay: Relay.AsObject): IRelayListHostname {
  return {
    ...relay,
    tunnels: relay.tunnels && {
      ...relay.tunnels,
      openvpn: relay.tunnels.openvpnList.map(convertOpenvpnList),
      wireguard: relay.tunnels.wireguardList.map(convertWireguardList),
    },
    bridges: relay.bridges && {
      shadowsocks: relay.bridges.shadowsocksList.map(convertShadowsocksList),
    },
  };
}

function convertOpenvpnList(openvpn: OpenVpnEndpointData.AsObject): IOpenVpnTunnelData {
  return {
    ...openvpn,
    protocol: convertTransportProtocol(openvpn.protocol),
  };
}

function convertWireguardList(wireguard: WireguardEndpointData.AsObject): IWireguardTunnelData {
  return {
    ...wireguard,
    portRanges: wireguard.portRangesList,
    publicKey: convertWireguardKey(wireguard.publicKey),
  };
}

function convertWireguardKey(publicKey: Uint8Array | string): string {
  return typeof publicKey === 'string' ? publicKey : new TextDecoder('utf-8').decode(publicKey);
}

function convertShadowsocksList(
  shadowsocks: ShadowsocksEndpointData.AsObject,
): IShadowsocksEndpointData {
  return {
    ...shadowsocks,
    protocol: convertTransportProtocol(shadowsocks.protocol),
  };
}

function convertTransportProtocol(protocol: TransportProtocol): RelayProtocol {
  const protocolMap: Record<TransportProtocol, RelayProtocol> = {
    [TransportProtocol.TCP]: 'tcp',
    [TransportProtocol.UDP]: 'udp',
    [TransportProtocol.ANY_PROTOCOL]: 'any',
  };
  return protocolMap[protocol];
}

function convertTunnelState(tunnelState: GrpcTunnelState): TunnelState | undefined {
  const tunnelStateObject = tunnelState.toObject();
  switch (tunnelState.getStateCase()) {
    case GrpcTunnelState.StateCase.STATE_NOT_SET:
      return undefined;
    case GrpcTunnelState.StateCase.DISCONNECTED:
      return { state: 'disconnected' };
    case GrpcTunnelState.StateCase.DISCONNECTING: {
      const detailsMap: Record<GrpcAfterDisconnect, AfterDisconnect> = {
        [GrpcAfterDisconnect.NOTHING]: 'nothing',
        [GrpcAfterDisconnect.BLOCK]: 'block',
        [GrpcAfterDisconnect.RECONNECT]: 'reconnect',
      };
      return (
        tunnelStateObject.disconnecting && {
          state: 'disconnecting',
          details: detailsMap[tunnelStateObject.disconnecting.afterDisconnect],
        }
      );
    }
    case GrpcTunnelState.StateCase.ERROR:
      return (
        tunnelStateObject.error?.errorState && {
          state: 'error',
          details: convertTunnelStateError(tunnelStateObject.error.errorState),
        }
      );
    case GrpcTunnelState.StateCase.CONNECTING:
      return {
        state: 'connecting',
        details:
          tunnelStateObject.connecting?.relayInfo &&
          convertTunnelStateRelayInfo(tunnelStateObject.connecting.relayInfo),
      };
    case GrpcTunnelState.StateCase.CONNECTED: {
      const relayInfo =
        tunnelStateObject.connected?.relayInfo &&
        convertTunnelStateRelayInfo(tunnelStateObject.connected.relayInfo);
      return (
        relayInfo && {
          state: 'connected',
          details: relayInfo,
        }
      );
    }
  }
}

function convertTunnelStateError(state: ErrorState.AsObject): IErrorState {
  return {
    ...state,
    cause: convertTunnelStateErrorCause(state.cause, state),
  };
}

function convertTunnelStateErrorCause(
  cause: ErrorState.Cause,
  state: ErrorState.AsObject,
): ErrorStateCause {
  switch (cause) {
    case ErrorState.Cause.IS_OFFLINE:
      return { reason: 'is_offline' };
    case ErrorState.Cause.SET_DNS_ERROR:
      return { reason: 'set_dns_error' };
    case ErrorState.Cause.IPV6_UNAVAILABLE:
      return { reason: 'ipv6_unavailable' };
    case ErrorState.Cause.START_TUNNEL_ERROR:
      return { reason: 'start_tunnel_error' };
    case ErrorState.Cause.TAP_ADAPTER_PROBLEM:
      return { reason: 'tap_adapter_problem' };
    case ErrorState.Cause.SET_FIREWALL_POLICY_ERROR:
      return { reason: 'set_firewall_policy_error' };
    case ErrorState.Cause.VPN_PERMISSION_DENIED:
      throw Error(); // TODO
    case ErrorState.Cause.AUTH_FAILED:
      return { reason: 'auth_failed', details: state.authFailReason };
    case ErrorState.Cause.TUNNEL_PARAMETER_ERROR: {
      const parameterErrorMap: Record<ErrorState.GenerationError, TunnelParameterError> = {
        [ErrorState.GenerationError.NO_MATCHING_RELAY]: 'no_matching_relay',
        [ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY]: 'no_matching_bridge_relay',
        [ErrorState.GenerationError.NO_WIREGUARD_KEY]: 'no_wireguard_key',
        [ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR]:
          'custom_tunnel_host_resultion_error',
      };
      return { reason: 'tunnel_parameter_error', details: parameterErrorMap[state.parameterError] };
    }
  }
}

function convertTunnelStateRelayInfo(
  state: TunnelStateRelayInfo.AsObject,
): ITunnelStateRelayInfo | undefined {
  return (
    state.tunnelEndpoint && {
      ...state,
      endpoint: {
        ...state.tunnelEndpoint,
        tunnelType: convertTunnelType(state.tunnelEndpoint.tunnelType),
        protocol: convertTransportProtocol(state.tunnelEndpoint.protocol),
        proxy: state.tunnelEndpoint.proxy && convertProxyEndpoint(state.tunnelEndpoint.proxy),
      },
    }
  );
}

function convertTunnelType(tunnelType: GrpcTunnelType): TunnelType {
  const tunnelTypeMap: Record<GrpcTunnelType, TunnelType> = {
    [GrpcTunnelType.ANY_TUNNEL]: 'any',
    [GrpcTunnelType.WIREGUARD]: 'wireguard',
    [GrpcTunnelType.OPENVPN]: 'openvpn',
  };

  return tunnelTypeMap[tunnelType];
}

function convertProxyEndpoint(proxyEndpoint: ProxyEndpoint.AsObject): IProxyEndpoint {
  const proxyTypeMap: Record<GrpcProxyType, ProxyType> = {
    [GrpcProxyType.CUSTOM]: 'custom',
    [GrpcProxyType.SHADOWSOCKS]: 'shadowsocks',
  };

  return {
    ...proxyEndpoint,
    protocol: convertTransportProtocol(proxyEndpoint.protocol),
    proxyType: proxyTypeMap[proxyEndpoint.proxyType],
  };
}

function convertSettings(_settings: Settings): ISettings | undefined {
  // TODO
  // const settingsObject = settings.toObject();
  // const bridgeState = settingsObject.bridgeState && convertBridgeState(settingsObject.bridgeState.state);
  // const relaySettings = convertRelaySettings(settings.getRelaySettings());
  // const bridgeSettings = settingsObject.bridgeSettings && convertBridgeSettings(settingsObject.bridgeSettings);
  // const tunnelOptions = settingsObject.tunnelOptions && convertTunnelOptions(settingsObject.tunnelOptions);
  return undefined;
  // return {
  //   ...settings.toObject(),
  //   bridgeState,
  //   relaySettings,
  //   bridgeSettings,
  //   tunnelOptions,
  // };
}

// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function convertBridgeState(bridgeState: GrpcBridgeState.State): BridgeState {
  const bridgeStateMap: Record<GrpcBridgeState.State, BridgeState> = {
    [GrpcBridgeState.State.AUTO]: 'auto',
    [GrpcBridgeState.State.ON]: 'on',
    [GrpcBridgeState.State.OFF]: 'off',
  };

  return bridgeStateMap[bridgeState];
}

// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function convertRelaySettings(relaySettings?: GrpcRelaySettings): RelaySettings | undefined {
  if (relaySettings) {
    switch (relaySettings.getEndpointCase()) {
      case GrpcRelaySettings.EndpointCase.ENDPOINT_NOT_SET:
        return undefined;
      case GrpcRelaySettings.EndpointCase.CUSTOM: {
        const custom = relaySettings.getCustom()?.toObject();
        const config = relaySettings.getCustom()?.getConfig();
        const connectionConfig = config && convertConnectionConfig(config);
        return (
          custom &&
          connectionConfig && {
            customTunnelEndpoint: {
              ...custom,
              config: connectionConfig,
            },
          }
        );
      }
      case GrpcRelaySettings.EndpointCase.NORMAL:
        // TODO
        return undefined;
    }
  } else {
    return undefined;
  }
}

function convertConnectionConfig(
  connectionConfig: GrpcConnectionConfig,
): ConnectionConfig | undefined {
  const connectionConfigObject = connectionConfig.toObject();
  switch (connectionConfig.getConfigCase()) {
    case GrpcConnectionConfig.ConfigCase.CONFIG_NOT_SET:
      return undefined;
    case GrpcConnectionConfig.ConfigCase.WIREGUARD:
      return (
        connectionConfigObject.wireguard &&
        connectionConfigObject.wireguard.tunnel &&
        connectionConfigObject.wireguard.peer && {
          wireguard: {
            ...connectionConfigObject.wireguard,
            tunnel: {
              privateKey: convertWireguardKey(connectionConfigObject.wireguard.tunnel.privateKey),
              addresses: connectionConfigObject.wireguard.tunnel.addressesList,
            },
            peer: {
              ...connectionConfigObject.wireguard.peer,
              addresses: connectionConfigObject.wireguard.peer.allowedIpsList,
              publicKey: convertWireguardKey(connectionConfigObject.wireguard.peer.publicKey),
            },
          },
        }
      );
    case GrpcConnectionConfig.ConfigCase.OPENVPN:
      return {
        openvpn: {
          ...connectionConfigObject.openvpn!,
          endpoint: {
            ip: connectionConfigObject.openvpn!.address,
            protocol: convertTransportProtocol(connectionConfigObject.openvpn!.protocol),
            port: 443, // TODO port is missing from OpenvpnConfig
          },
        },
      };
  }
}

// function convertBridgeSettings(bridgeSettings: GrpcBridgeSettings.AsObject): BridgeSettings {
// }

// function convertTunnelOptions(tunnelOptions: TunnelOptions.AsObject): ITunnelOptions {
// }
