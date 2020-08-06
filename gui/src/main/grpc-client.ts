import * as grpc from '@grpc/grpc-js';
import {
  BoolValue,
  StringValue,
  UInt32Value,
} from 'google-protobuf/google/protobuf/wrappers_pb.js';
import log from 'electron-log';
import { Empty } from 'google-protobuf/google/protobuf/empty_pb.js';
import { promisify } from 'util';
import {
  AccountToken,
  Constraint,
  IRelayListCountry,
  IRelayListCity,
  IRelayListHostname,
  IWireguardTunnelData,
  IBridgeConstraints,
  IWireguardConstraints,
  ITunnelOptions,
  IOpenVpnConstraints,
  IShadowsocksEndpointData,
  RelayProtocol,
  BridgeSettings,
  FirewallPolicyError,
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
  DaemonEvent,
  RelaySettingsNormalUpdate,
  RelaySettings,
  RelayLocation,
  ProxySettings,
  TunnelProtocol,
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
  NormalRelaySettingsUpdate,
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
import consumePromise from '../shared/promise';
import * as grpcTypes from './management_interface/management_interface_pb';

const NETWORK_CALL_TIMEOUT = 10000;
const CHANNEL_STATE_TIMEOUT = 1000 * 60 * 60;

export interface ErrorResponse {
  code: number;
  details: string;
}

const ManagementServiceClient = grpc.makeClientConstructor(
  // @ts-ignore
  managementInterface['mullvad_daemon.management_interface.ManagementService'],
  'ManagementService',
);

const noConnectionError = new Error('No connection established to daemon');
const configNotSupported = new Error('Setting custom settings is not supported');

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
  public subscriptionId?: number;

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
  constructor(connectionParams: string) {
    this.client = (new ManagementServiceClient(
      connectionParams,
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    ) as unknown) as managementInterface.ManagementServiceClient;
  }

  private client: managementInterface.ManagementServiceClient;
  private isConnected = false;
  private connectionObservers: ConnectionObserver[] = [];
  private nextSubscriptionId = 0;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private subscriptions: Map<number, grpc.ClientReadableStream<any>> = new Map();
  private connectionPromise?: Promise<void>;

  private subscriptionId(): number {
    const current = this.nextSubscriptionId;
    this.nextSubscriptionId += 1;
    return current;
  }

  private deadlineFromNow() {
    return Date.now() + NETWORK_CALL_TIMEOUT;
  }

  private channelStateTimeout(): number {
    return Date.now() + CHANNEL_STATE_TIMEOUT;
  }

  private callEmpty<R>(fn: CallFunctionArgument<Empty, R>): Promise<R> {
    return this.call<Empty, R>(fn, new Empty());
  }

  private callString<R>(fn: CallFunctionArgument<StringValue, R>, value?: string): Promise<R> {
    const googleString = new StringValue();

    if (value !== undefined) {
      googleString.setValue(value);
    }

    return this.call<StringValue, R>(fn, googleString);
  }

  private callBool<R>(fn: CallFunctionArgument<BoolValue, R>, value?: boolean): Promise<R> {
    const googleBool = new BoolValue();

    if (value !== undefined) {
      googleBool.setValue(value);
    }

    return this.call<BoolValue, R>(fn, googleBool);
  }

  private callNumber<R>(fn: CallFunctionArgument<UInt32Value, R>, value?: number): Promise<R> {
    const googleNumber = new UInt32Value();

    if (value !== undefined) {
      googleNumber.setValue(value);
    }

    return this.call<UInt32Value, R>(fn, googleNumber);
  }

  private call<T, R>(fn: CallFunctionArgument<T, R>, arg: T): Promise<R> {
    if (this.client && fn && this.isConnected) {
      return promisify<T, R>(fn.bind(this.client))(arg);
    } else {
      throw noConnectionError;
    }
  }

  public connect(): Promise<void> {
    if (this.connectionPromise === undefined) {
      this.connectionPromise = new Promise((resolve, reject) => {
        this.client.waitForReady(this.deadlineFromNow(), (error) => {
          this.connectionPromise = undefined;
          if (error) {
            this.client.getChannel()?.getConnectivityState(false);
            this.connectionObservers.forEach((observer) => observer.onClose(error));
            this.ensureConnectivity();
            reject(error);
          } else {
            this.isConnected = true;
            this.connectionObservers.forEach((observer) => observer.onOpen());
            this.setChannelCallback();
            resolve();
          }
        });
      });
    }
    return this.connectionPromise;
  }

  private channelOptions(): grpc.ClientOptions {
    return {
      'grpc.max_reconnect_backoff_ms': 3000,
      'grpc.initial_reconnect_backoff_ms': 3000,
      'grpc.keepalive_time_ms': Math.pow(2, 30),
      'grpc.keepalive_timeout_ms': Math.pow(2, 30),
    };
  }

  private connectivityChangeCallback(timeoutErr?: Error) {
    const channel = this.client.getChannel();
    const currentState = channel?.getConnectivityState(true);
    log.debug(`GRPC Channel connectivity state changed to ${currentState}`);
    if (channel) {
      if (timeoutErr) {
        this.setChannelCallback(currentState);
        return;
      }
      const wasConnected = this.isConnected;
      if (this.channelDisconnected(currentState)) {
        this.connectionObservers.forEach((observer) => observer.onClose());
        this.isConnected = false;
        // Try and reconnect in case
        consumePromise(
          this.connect().catch((error) => {
            log.error(`Failed to reconnect - ${error}`);
          }),
        );
      } else if (!wasConnected) {
        this.isConnected = true;
        this.connectionObservers.forEach((observer) => observer.onOpen());
      }
      this.setChannelCallback(currentState);
    }
  }

  private channelDisconnected(state: grpc.connectivityState): boolean {
    return (
      (state == grpc.connectivityState.SHUTDOWN ||
        state == grpc.connectivityState.TRANSIENT_FAILURE ||
        state == grpc.connectivityState.IDLE) &&
      this.isConnected
    );
  }

  private setChannelCallback(currentState?: grpc.connectivityState) {
    const channel = this.client?.getChannel();
    if (currentState === undefined && channel) {
      currentState = channel?.getConnectivityState(false);
    }
    if (currentState) {
      channel.watchConnectivityState(currentState, this.channelStateTimeout(), (error) =>
        this.connectivityChangeCallback(error),
      );
    }
  }

  // Since grpc.Channel.watchConnectivityState() isn't always running as intended, whenever the
  // client fails to connect at first, `ensureConnectivity()` should be called so that it tries to
  // check the connectivity state and nudge the client into connecting.
  // `grpc.Channel.getConnectivityState(true)` should make it attempt to connect.
  private ensureConnectivity(lastState?: grpc.connectivityState) {
    setTimeout(() => {
      if (this.client) {
        if (lastState === undefined) {
          lastState = this.client.getChannel().getConnectivityState(true);
        }
        if (this.channelDisconnected(lastState)) {
          this.connectionObservers.forEach((observer) => observer.onClose());
          this.isConnected = false;
        }
        if (!this.isConnected) {
          consumePromise(
            this.connect().catch((error) => {
              log.error(`Failed to reconnect - ${error}`);
            }),
          );
        }
      }
    }, 3000);
  }

  public disconnect() {
    this.isConnected = false;
    this.subscriptions.clear();
    this.client?.close();
  }

  public addConnectionObserver(observer: ConnectionObserver) {
    this.connectionObservers.push(observer);
    const currentState = this.client.getChannel()?.getConnectivityState(true);
    if (
      currentState == grpc.connectivityState.SHUTDOWN ||
      currentState == grpc.connectivityState.TRANSIENT_FAILURE ||
      currentState == grpc.connectivityState.IDLE
    ) {
      observer.onClose();
    } else {
      observer.onOpen();
    }
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
          relayLocations.push(convertFromRelayListCountry(country.toObject())),
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

  // TODO: Custom tunnel configurations are not supported by the GUI.
  public async updateRelaySettings(relaySettings: RelaySettingsNormalUpdate): Promise<void> {
    const grpcRelaySettings = new grpcTypes.RelaySettingsUpdate();

    const normalUpdate = new NormalRelaySettingsUpdate();
    const tunnelTypeUpdate = new grpcTypes.TunnelTypeUpdate();
    tunnelTypeUpdate.setTunnelType(convertToTunnelType(relaySettings.tunnelProtocol));
    normalUpdate.setLocation(convertToLocation(liftConstraint(relaySettings.location)));
    normalUpdate.setTunnelType(tunnelTypeUpdate);
    normalUpdate.setWireguardConstraints(
      convertToWireguardConstraints(relaySettings.wireguardConstraints),
    );
    normalUpdate.setOpenvpnConstraints(
      convertToOpenVpnConstraints(relaySettings.openvpnConstraints),
    );

    grpcRelaySettings.setNormal(normalUpdate);
    await this.call<grpcTypes.RelaySettingsUpdate, Empty>(
      this.client?.updateRelaySettings,
      grpcRelaySettings,
    );
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

  public async setBridgeSettings(bridgeSettings: BridgeSettings): Promise<void> {
    const grpcBridgeSettings = new grpcTypes.BridgeSettings();

    if ('normal' in bridgeSettings) {
      const normalSettings = convertToNormalBridgeSettings(bridgeSettings.normal);
      grpcBridgeSettings.setNormal(normalSettings);
    }

    if ('custom' in bridgeSettings) {
      throw configNotSupported;
    }

    await this.call<grpcTypes.BridgeSettings, Empty>(
      this.client?.setBridgeSettings,
      grpcBridgeSettings,
    );
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
    return convertFromTunnelState(response)!;
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.callEmpty<Settings>(this.client?.getSettings);
    return convertFromSettings(response)!;
  }

  public subscribeDaemonEventListener(listener: SubscriptionListener<DaemonEvent>) {
    const call = this.isConnected && this.client?.eventsListen(new Empty());
    if (!call) {
      throw noConnectionError;
    }
    const subscriptionId = this.subscriptionId();
    listener.subscriptionId = subscriptionId;

    call.on('data', (data: grpcTypes.DaemonEvent) => {
      try {
        const daemonEvent = convertFromDaemonEvent(data);
        listener.onEvent(daemonEvent);
      } catch (err) {
        listener.onError(err);
      }
    });

    const removeSubscription = () => {
      const subscription = this.subscriptions.get(subscriptionId);
      if (subscription !== undefined) {
        subscription.cancel();
        this.subscriptions.delete(subscriptionId);
      }
    };

    call.on('error', (error) => {
      listener.onError(error);
      removeSubscription();
    });
  }

  public unsubscribeDaemonEventListener(listener: SubscriptionListener<DaemonEvent>) {
    const id = listener.subscriptionId;
    if (id !== undefined) {
      const subscription = this.subscriptions.get(id);
      if (subscription !== undefined) {
        subscription.cancel();
        this.subscriptions.delete(id);
      }
    }
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
    return convertFromKeygenEvent(response);
  }

  public async getWireguardKey(): Promise<IWireguardPublicKey> {
    const response = await this.callEmpty<PublicKey>(this.client?.getWireguardKey);
    return {
      created: response.getCreated()!.toDate().toISOString(),
      key: convertFromWireguardKey(response.getKey()),
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

function convertFromRelayListCountry(country: RelayListCountry.AsObject): IRelayListCountry {
  return {
    ...country,
    cities: country.citiesList.map(convertFromRelayListCity),
  };
}

function convertFromRelayListCity(city: RelayListCity.AsObject): IRelayListCity {
  return {
    ...city,
    relays: city.relaysList.map(convertFromRelayListRelay),
  };
}

function convertFromRelayListRelay(relay: Relay.AsObject): IRelayListHostname {
  return {
    ...relay,
    tunnels: relay.tunnels && {
      ...relay.tunnels,
      openvpn: relay.tunnels.openvpnList.map(convertFromOpenvpnList),
      wireguard: relay.tunnels.wireguardList.map(convertFromWireguardList),
    },
    bridges: relay.bridges && {
      shadowsocks: relay.bridges.shadowsocksList.map(convertFromShadowsocksList),
    },
  };
}

function convertFromOpenvpnList(openvpn: OpenVpnEndpointData.AsObject): IOpenVpnTunnelData {
  return {
    ...openvpn,
    protocol: convertFromTransportProtocol(openvpn.protocol),
  };
}

function convertFromWireguardList(wireguard: WireguardEndpointData.AsObject): IWireguardTunnelData {
  return {
    ...wireguard,
    portRanges: wireguard.portRangesList,
    publicKey: convertFromWireguardKey(wireguard.publicKey),
  };
}

function convertFromWireguardKey(publicKey: Uint8Array | string): string {
  /// I doubt this will ever work.
  if (typeof publicKey === 'string') {
    return publicKey;
  }
  return Buffer.from(publicKey).toString('base64');
}

function convertFromShadowsocksList(
  shadowsocks: ShadowsocksEndpointData.AsObject,
): IShadowsocksEndpointData {
  return {
    ...shadowsocks,
    protocol: convertFromTransportProtocol(shadowsocks.protocol),
  };
}

function convertFromTransportProtocol(protocol: TransportProtocol): RelayProtocol {
  const protocolMap: Record<TransportProtocol, RelayProtocol> = {
    [TransportProtocol.TCP]: 'tcp',
    [TransportProtocol.UDP]: 'udp',
    [TransportProtocol.ANY_PROTOCOL]: 'any',
  };
  return protocolMap[protocol];
}

function convertFromTunnelState(tunnelState: GrpcTunnelState): TunnelState | undefined {
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
          details: convertFromTunnelStateError(tunnelStateObject.error.errorState),
        }
      );
    case GrpcTunnelState.StateCase.CONNECTING:
      return {
        state: 'connecting',
        details:
          tunnelStateObject.connecting?.relayInfo &&
          convertFromTunnelStateRelayInfo(tunnelStateObject.connecting.relayInfo),
      };
    case GrpcTunnelState.StateCase.CONNECTED: {
      const relayInfo =
        tunnelStateObject.connected?.relayInfo &&
        convertFromTunnelStateRelayInfo(tunnelStateObject.connected.relayInfo);
      return (
        relayInfo && {
          state: 'connected',
          details: relayInfo,
        }
      );
    }
  }
}

function convertFromTunnelStateError(state: ErrorState.AsObject): IErrorState {
  return {
    ...state,
    cause: convertFromTunnelStateErrorCause(state.cause, state),
  };
}

function convertFromTunnelStateErrorCause(
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
      return {
        reason: 'set_firewall_policy_error',
        details: convertFromFirewallPolicyError(state.policyError!),
      };
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

function convertFromFirewallPolicyError(
  error: grpcTypes.ErrorState.FirewallPolicyError.AsObject,
): FirewallPolicyError {
  switch (error.type) {
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.GENERIC:
      return { reason: 'generic' };
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.LOCKED:
      const pid = error.lockPid;
      const name = error.lockName;
      return { reason: 'locked', details: pid && name ? { pid, name } : undefined };
    default:
      throw new Error('unreachable');
  }
}

function convertFromTunnelStateRelayInfo(
  state: TunnelStateRelayInfo.AsObject,
): ITunnelStateRelayInfo | undefined {
  return (
    state.tunnelEndpoint && {
      ...state,
      endpoint: {
        ...state.tunnelEndpoint,
        tunnelType: convertFromTunnelType(state.tunnelEndpoint.tunnelType),
        protocol: convertFromTransportProtocol(state.tunnelEndpoint.protocol),
        proxy: state.tunnelEndpoint.proxy && convertFromProxyEndpoint(state.tunnelEndpoint.proxy),
      },
    }
  );
}

function convertFromTunnelType(tunnelType: GrpcTunnelType): TunnelType {
  const tunnelTypeMap: Record<GrpcTunnelType, TunnelType> = {
    [GrpcTunnelType.ANY_TUNNEL]: 'any',
    [GrpcTunnelType.WIREGUARD]: 'wireguard',
    [GrpcTunnelType.OPENVPN]: 'openvpn',
  };

  return tunnelTypeMap[tunnelType];
}

function convertFromProxyEndpoint(proxyEndpoint: ProxyEndpoint.AsObject): IProxyEndpoint {
  const proxyTypeMap: Record<GrpcProxyType, ProxyType> = {
    [GrpcProxyType.CUSTOM]: 'custom',
    [GrpcProxyType.SHADOWSOCKS]: 'shadowsocks',
  };

  return {
    ...proxyEndpoint,
    protocol: convertFromTransportProtocol(proxyEndpoint.protocol),
    proxyType: proxyTypeMap[proxyEndpoint.proxyType],
  };
}

function convertFromSettings(settings: Settings): ISettings | undefined {
  const settingsObject = settings.toObject();
  const bridgeState = convertFromBridgeState(settingsObject.bridgeState!.state!);
  const relaySettings = convertFromRelaySettings(settings.getRelaySettings())!;
  const bridgeSettings = convertFromBridgeSettings(settingsObject.bridgeSettings!);
  const tunnelOptions = convertFromTunnelOptions(settingsObject.tunnelOptions!);
  return {
    ...settings.toObject(),
    bridgeState,
    relaySettings,
    bridgeSettings,
    tunnelOptions,
  };
}

function convertFromBridgeState(bridgeState: GrpcBridgeState.State): BridgeState {
  const bridgeStateMap: Record<GrpcBridgeState.State, BridgeState> = {
    [GrpcBridgeState.State.AUTO]: 'auto',
    [GrpcBridgeState.State.ON]: 'on',
    [GrpcBridgeState.State.OFF]: 'off',
  };

  return bridgeStateMap[bridgeState];
}

function convertFromRelaySettings(relaySettings?: GrpcRelaySettings): RelaySettings | undefined {
  /*  eslint-disable no-case-declarations */
  if (relaySettings) {
    switch (relaySettings.getEndpointCase()) {
      case GrpcRelaySettings.EndpointCase.ENDPOINT_NOT_SET:
        return undefined;
      case GrpcRelaySettings.EndpointCase.CUSTOM: {
        const custom = relaySettings.getCustom()?.toObject();
        const config = relaySettings.getCustom()?.getConfig();
        const connectionConfig = config && convertFromConnectionConfig(config);
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
        const normal = relaySettings.getNormal()!;
        const grpcLocation = normal.getLocation();
        const location = grpcLocation
          ? { only: convertFromLocation(grpcLocation.toObject()) }
          : 'any';
        const tunnelProtocol = convertFromTunnelTypeConstraint(normal.getTunnelType()!);
        const openvpnConstraints = convertFromOpenVpnConstraints(normal.getOpenvpnConstraints()!);
        const wireguardConstraints = convertFromWireguardConstraints(
          normal.getWireguardConstraints()!,
        );

        return {
          normal: {
            location,
            tunnelProtocol,
            wireguardConstraints,
            openvpnConstraints,
          },
        };
    }
  } else {
    return undefined;
  }
}

function convertFromBridgeSettings(
  bridgeSettings: grpcTypes.BridgeSettings.AsObject,
): BridgeSettings {
  const normalSettings = bridgeSettings.normal;
  if (normalSettings) {
    const grpcLocation = normalSettings.location;
    const location = grpcLocation ? { only: convertFromLocation(grpcLocation) } : 'any';
    return {
      normal: {
        location,
      },
    };
  }

  const customSettings = (settings: ProxySettings): BridgeSettings => {
    return { custom: settings };
  };

  const localSettings = bridgeSettings.local;
  if (localSettings) {
    return customSettings({
      port: localSettings?.port!,
      peer: localSettings?.peer!,
    });
  }

  const remoteSettings = bridgeSettings.remote;
  if (remoteSettings) {
    return customSettings({
      address: remoteSettings?.address!,
      auth: {
        ...remoteSettings?.auth!,
      },
    });
  }

  const shadowsocksSettings = bridgeSettings.shadowsocks!;
  return customSettings({
    peer: shadowsocksSettings.peer!,
    password: shadowsocksSettings.password!,
    cipher: shadowsocksSettings.cipher!,
  });
}

function convertFromConnectionConfig(
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
              privateKey: convertFromWireguardKey(
                connectionConfigObject.wireguard.tunnel.privateKey,
              ),
              addresses: connectionConfigObject.wireguard.tunnel.addressesList,
            },
            peer: {
              ...connectionConfigObject.wireguard.peer,
              addresses: connectionConfigObject.wireguard.peer.allowedIpsList,
              publicKey: convertFromWireguardKey(connectionConfigObject.wireguard.peer.publicKey),
            },
          },
        }
      );
    case GrpcConnectionConfig.ConfigCase.OPENVPN:
      // eslint-disable-next-line no-case-declarations
      const [ip, port] = connectionConfigObject.openvpn!.address.split(':');
      return {
        openvpn: {
          ...connectionConfigObject.openvpn!,
          endpoint: {
            ip,
            port: parseInt(port, 10),
            protocol: convertFromTransportProtocol(connectionConfigObject.openvpn!.protocol),
          },
        },
      };
  }
}

function convertFromLocation(location: grpcTypes.RelayLocation.AsObject): RelayLocation {
  if (location.hostname) {
    return { hostname: [location.country, location.city, location.hostname] };
  }
  if (location.city) {
    return { city: [location.country, location.city] };
  }

  return { country: location.country };
}

function convertFromTunnelOptions(tunnelOptions: grpcTypes.TunnelOptions.AsObject): ITunnelOptions {
  return {
    openvpn: {
      mssfix: tunnelOptions.openvpn!.mssfix,
    },
    wireguard: {
      mtu: tunnelOptions.wireguard!.mtu,
    },
    generic: {
      enableIpv6: tunnelOptions.generic!.enableIpv6,
    },
  };
}

function convertFromDaemonEvent(data: grpcTypes.DaemonEvent): DaemonEvent {
  const tunnelState = data.getTunnelState();
  if (tunnelState !== undefined) {
    return { tunnelState: convertFromTunnelState(tunnelState)! };
  }

  const settings = data.getSettings();
  if (settings !== undefined) {
    return { settings: convertFromSettings(settings)! };
  }

  const relayList = data.getRelayList();
  if (relayList !== undefined) {
    return {
      relayList: {
        countries: relayList
          .getCountriesList()
          ?.map((country: grpcTypes.RelayListCountry) =>
            convertFromRelayListCountry(country.toObject()),
          ),
      },
    };
  }

  const keygenEvent = data.getKeyEvent();
  if (keygenEvent !== undefined) {
    return {
      wireguardKey: convertFromKeygenEvent(keygenEvent),
    };
  }

  return {
    appVersionInfo: data.getVersionInfo()!.toObject(),
  };
}

function convertFromKeygenEvent(data: grpcTypes.KeygenEvent): KeygenEvent {
  switch (data.getEvent()) {
    case GrpcKeygenEvent.KeygenEvent.TOO_MANY_KEYS:
      return 'too_many_keys';
    case GrpcKeygenEvent.KeygenEvent.NEW_KEY: {
      const newKey = data.getNewKey();
      return newKey
        ? {
            newKey: {
              created: newKey.getCreated()!.toDate().toISOString(),
              key: convertFromWireguardKey(newKey.getKey()),
            },
          }
        : 'generation_failure';
    }
    case GrpcKeygenEvent.KeygenEvent.GENERATION_FAILURE:
      return 'generation_failure';
  }
}

function convertFromOpenVpnConstraints(
  constraints: grpcTypes.OpenvpnConstraints,
): IOpenVpnConstraints {
  const port = convertFromConstraint(constraints.getPort());
  let protocol: Constraint<RelayProtocol> = 'any';
  switch (constraints.getProtocol()) {
    case grpcTypes.TransportProtocol.TCP:
      protocol = { only: 'tcp' };
      break;
    case grpcTypes.TransportProtocol.UDP:
      protocol = { only: 'udp' };
      break;
  }

  return { port, protocol };
}

function convertFromWireguardConstraints(
  constraints: grpcTypes.WireguardConstraints,
): IWireguardConstraints {
  const port = convertFromConstraint(constraints.getPort());
  return { port };
}

function convertFromTunnelTypeConstraint(
  tunnelType: grpcTypes.TunnelType,
): Constraint<TunnelProtocol> {
  switch (tunnelType) {
    case grpcTypes.TunnelType.ANY_TUNNEL: {
      return 'any';
    }
    case grpcTypes.TunnelType.WIREGUARD: {
      return { only: 'wireguard' };
    }
    case grpcTypes.TunnelType.OPENVPN: {
      return { only: 'openvpn' };
    }
  }
}

function convertFromConstraint<T>(value: T | undefined): Constraint<T> {
  if (value !== undefined) {
    return { only: value };
  } else {
    return 'any';
  }
}

function convertToNormalBridgeSettings(
  constraints: IBridgeConstraints,
): grpcTypes.BridgeSettings.BridgeConstraints {
  const normalBridgeSettings = new grpcTypes.BridgeSettings.BridgeConstraints();
  normalBridgeSettings.setLocation(convertToLocation(liftConstraint(constraints.location)));

  return normalBridgeSettings;
}

function convertToLocation(constraint: RelayLocation | undefined): grpcTypes.RelayLocation | undefined {
  const location = new grpcTypes.RelayLocation();
  if (constraint && 'hostname' in constraint) {
    const [countryCode, cityCode, hostname] = constraint.hostname;
    location.setCountry(countryCode);
    location.setCity(cityCode);
    location.setHostname(hostname);
    return location;
  } else if (constraint && 'city' in constraint) {
    location.setCountry(constraint.city[0]);
    location.setCity(constraint.city[1]);
    return location;
  } else if (constraint && 'country' in constraint) {
    location.setCity(constraint.country);
    return location;
  } else {
    return undefined;
  }
}

function convertToTunnelType(constraint: Constraint<TunnelType> | undefined): GrpcTunnelType {
  if (constraint !== undefined && constraint !== 'any' && 'only' in constraint) {
    switch (constraint.only) {
      case 'wireguard':
        return GrpcTunnelType.WIREGUARD;
      case 'openvpn':
        return GrpcTunnelType.OPENVPN;
      default:
        return GrpcTunnelType.ANY_TUNNEL;
    }
  }
  return GrpcTunnelType.ANY_TUNNEL;
}

function convertToOpenVpnConstraints(
  constraints: Partial<IOpenVpnConstraints> | undefined,
): grpcTypes.OpenvpnConstraints | undefined {
  const openvpnConstraints = new grpcTypes.OpenvpnConstraints();
  if (constraints) {
    const port = liftConstraint(constraints.port);
    if (port) {
      openvpnConstraints.setPort(port);
    }
    const protocol = liftConstraint(constraints.protocol);
    if (protocol) {
      openvpnConstraints.setProtocol(convertToTransportProtocol(protocol));
    }
    return openvpnConstraints;
  }

  return undefined;
}

function convertToWireguardConstraints(
  constraint: Partial<IWireguardConstraints> | undefined,
): grpcTypes.WireguardConstraints | undefined {
  const wireguardConstraints = new grpcTypes.WireguardConstraints();
  const port = constraint ? liftConstraint(constraint.port) : undefined;
  if (port !== undefined) {
    wireguardConstraints.setPort(port);
    return wireguardConstraints;
  }

  return undefined;
}

function convertToTransportProtocol(protocol: RelayProtocol): TransportProtocol {
  switch (protocol) {
    case 'any':
      return TransportProtocol.ANY_PROTOCOL;
    case 'udp':
      return TransportProtocol.UDP;
    case 'tcp':
      return TransportProtocol.TCP;
  }
}

function liftConstraint<T>(constraint: Constraint<T> | undefined): T | undefined {
  if (constraint !== undefined && constraint !== 'any') {
    return constraint.only;
  }
  return undefined;
}
