import * as grpc from '@grpc/grpc-js';
import { Empty } from 'google-protobuf/google/protobuf/empty_pb.js';
import {
  BoolValue,
  StringValue,
  UInt32Value,
} from 'google-protobuf/google/protobuf/wrappers_pb.js';
import { promisify } from 'util';

import {
  AccountDataError,
  AccountDataResponse,
  AccountToken,
  AfterDisconnect,
  AuthFailedError,
  BridgeSettings,
  BridgeState,
  ConnectionConfig,
  Constraint,
  CustomListError,
  CustomLists,
  DaemonEvent,
  DeviceEvent,
  DeviceState,
  EndpointObfuscationType,
  ErrorState,
  ErrorStateCause,
  FirewallPolicyError,
  FirewallPolicyErrorType,
  IAppVersionInfo,
  IBridgeConstraints,
  ICustomList,
  IDevice,
  IDeviceRemoval,
  IDnsOptions,
  ILocation,
  IObfuscationEndpoint,
  IOpenVpnConstraints,
  IProxyEndpoint,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  IRelayListWithEndpointData,
  IRelaySettingsNormal,
  ISettings,
  ITunnelOptions,
  ITunnelStateRelayInfo,
  IWireguardConstraints,
  IWireguardEndpointData,
  LoggedInDeviceState,
  LoggedOutDeviceState,
  ObfuscationSettings,
  ObfuscationType,
  Ownership,
  ProxySettings,
  ProxyType,
  RelayEndpointType,
  RelayLocation,
  RelayLocationGeographical,
  RelayProtocol,
  RelaySettings,
  TunnelParameterError,
  TunnelProtocol,
  TunnelState,
  TunnelType,
  VoucherResponse,
  wrapConstraint,
} from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import { ManagementServiceClient } from './management_interface/management_interface_grpc_pb';
import * as grpcTypes from './management_interface/management_interface_pb';

const DAEMON_RPC_PATH =
  process.platform === 'win32' ? 'unix:////./pipe/Mullvad VPN' : 'unix:///var/run/mullvad-vpn';

const NETWORK_CALL_TIMEOUT = 10000;
const CHANNEL_STATE_TIMEOUT = 1000 * 60 * 60;

const noConnectionError = new Error('No connection established to daemon');
const configNotSupported = new Error('Setting custom settings is not supported');
const invalidErrorStateCause = new Error(
  'VPN_PERMISSION_DENIED is not a valid error state cause on desktop',
);

export class ConnectionObserver {
  constructor(
    private openHandler: () => void,
    private closeHandler: (wasConnected: boolean, error?: Error) => void,
  ) {}

  // Only meant to be called by DaemonRpc
  // @internal
  public onOpen = () => {
    this.openHandler();
  };

  // Only meant to be called by DaemonRpc
  // @internal
  public onClose = (wasConnected: boolean, error?: Error) => {
    this.closeHandler(wasConnected, error);
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

export class ResponseParseError extends Error {
  constructor(message: string) {
    super(message);
  }
}

type CallFunctionArgument<T, R> =
  | ((arg: T, callback: (error: Error | null, result: R) => void) => void)
  | undefined;

export class DaemonRpc {
  private client: ManagementServiceClient;
  private isConnectedValue = false;
  private isClosed = false;
  private nextSubscriptionId = 0;
  private subscriptions: Map<number, grpc.ClientReadableStream<grpcTypes.DaemonEvent>> = new Map();
  private reconnectionTimeout?: NodeJS.Timer;

  constructor(private connectionObserver?: ConnectionObserver) {
    this.client = new ManagementServiceClient(
      DAEMON_RPC_PATH,
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    );
  }

  public get isConnected() {
    return this.isConnectedValue;
  }

  public reopen(connectionObserver?: ConnectionObserver) {
    if (this.isClosed) {
      this.isClosed = false;
      this.client = new ManagementServiceClient(
        DAEMON_RPC_PATH,
        grpc.credentials.createInsecure(),
        this.channelOptions(),
      );

      this.connectionObserver = connectionObserver;
    }
  }

  public connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.client.waitForReady(this.deadlineFromNow(), (error) => {
        if (error) {
          this.onClose(error);
          this.ensureConnectivity();
          reject(error);
        } else {
          this.reconnectionTimeout = undefined;
          this.isConnectedValue = true;
          this.connectionObserver?.onOpen();
          this.setChannelCallback();
          resolve();
        }
      });
    });
  }

  public disconnect() {
    this.isConnectedValue = false;

    for (const subscriptionId of this.subscriptions.keys()) {
      this.removeSubscription(subscriptionId);
    }

    this.isClosed = true;
    this.client.close();
    this.connectionObserver = undefined;
    if (this.reconnectionTimeout) {
      clearTimeout(this.reconnectionTimeout);
    }
  }

  public async getAccountData(accountToken: AccountToken): Promise<AccountDataResponse> {
    try {
      const response = await this.callString<grpcTypes.AccountData>(
        this.client.getAccountData,
        accountToken,
      );
      const expiry = response.getExpiry()!.toDate().toISOString();
      return { type: 'success', expiry };
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code) {
        switch (error.code) {
          case grpc.status.UNAUTHENTICATED:
            return { type: 'error', error: 'invalid-account' };
          default:
            return { type: 'error', error: 'communication' };
        }
      }
      throw error;
    }
  }

  public async getWwwAuthToken(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client.getWwwAuthToken);
    return response.getValue();
  }

  public async submitVoucher(voucherCode: string): Promise<VoucherResponse> {
    try {
      const response = await this.callString<grpcTypes.VoucherSubmission>(
        this.client.submitVoucher,
        voucherCode,
      );

      const secondsAdded = ensureExists(
        response.getSecondsAdded(),
        "no 'secondsAdded' field in voucher response",
      );
      const newExpiry = ensureExists(
        response.getNewExpiry(),
        "no 'newExpiry' field in voucher response",
      )
        .toDate()
        .toISOString();
      return {
        type: 'success',
        secondsAdded,
        newExpiry,
      };
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code) {
        switch (error.code) {
          case grpc.status.NOT_FOUND:
            return { type: 'invalid' };
          case grpc.status.RESOURCE_EXHAUSTED:
            return { type: 'already_used' };
        }
      }
      return { type: 'error' };
    }
  }

  public async getRelayLocations(): Promise<IRelayListWithEndpointData> {
    if (this.isConnected) {
      const response = await this.callEmpty<grpcTypes.RelayList>(this.client.getRelayLocations);
      return convertFromRelayList(response);
    } else {
      throw noConnectionError;
    }
  }

  public async createNewAccount(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client.createNewAccount);
    return response.getValue();
  }

  public async loginAccount(accountToken: AccountToken): Promise<AccountDataError | void> {
    try {
      await this.callString(this.client.loginAccount, accountToken);
    } catch (e) {
      const error = e as grpc.ServiceError;
      switch (error.code) {
        case grpc.status.RESOURCE_EXHAUSTED:
          return { type: 'error', error: 'too-many-devices' };
        case grpc.status.UNAUTHENTICATED:
          return { type: 'error', error: 'invalid-account' };
        default:
          return { type: 'error', error: 'communication' };
      }
    }
  }

  public async logoutAccount(): Promise<void> {
    await this.callEmpty(this.client.logoutAccount);
  }

  // TODO: Custom tunnel configurations are not supported by the GUI.
  public async updateRelaySettings(relaySettings: RelaySettings): Promise<void> {
    if ('normal' in relaySettings) {
      const normalSettings = relaySettings.normal;
      const grpcRelaySettings = new grpcTypes.RelaySettings();
      grpcRelaySettings.setNormal(convertToRelayConstraints(normalSettings));

      await this.call<grpcTypes.RelaySettings, Empty>(
        this.client.setRelaySettings,
        grpcRelaySettings,
      );
    }
  }

  public async setAllowLan(allowLan: boolean): Promise<void> {
    await this.callBool(this.client.setAllowLan, allowLan);
  }

  public async setShowBetaReleases(showBetaReleases: boolean): Promise<void> {
    await this.callBool(this.client.setShowBetaReleases, showBetaReleases);
  }

  public async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this.callBool(this.client.setEnableIpv6, enableIpv6);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this.callBool(this.client.setBlockWhenDisconnected, blockWhenDisconnected);
  }

  public async setBridgeState(bridgeState: BridgeState): Promise<void> {
    const bridgeStateMap = {
      auto: grpcTypes.BridgeState.State.AUTO,
      on: grpcTypes.BridgeState.State.ON,
      off: grpcTypes.BridgeState.State.OFF,
    };

    const grpcBridgeState = new grpcTypes.BridgeState();
    grpcBridgeState.setState(bridgeStateMap[bridgeState]);
    await this.call<grpcTypes.BridgeState, Empty>(this.client.setBridgeState, grpcBridgeState);
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
      this.client.setBridgeSettings,
      grpcBridgeSettings,
    );
  }

  public async setObfuscationSettings(obfuscationSettings: ObfuscationSettings): Promise<void> {
    const grpcObfuscationSettings = new grpcTypes.ObfuscationSettings();
    switch (obfuscationSettings.selectedObfuscation) {
      case ObfuscationType.auto:
        grpcObfuscationSettings.setSelectedObfuscation(
          grpcTypes.ObfuscationSettings.SelectedObfuscation.AUTO,
        );
        break;
      case ObfuscationType.off:
        grpcObfuscationSettings.setSelectedObfuscation(
          grpcTypes.ObfuscationSettings.SelectedObfuscation.OFF,
        );
        break;
      case ObfuscationType.udp2tcp:
        grpcObfuscationSettings.setSelectedObfuscation(
          grpcTypes.ObfuscationSettings.SelectedObfuscation.UDP2TCP,
        );
        break;
    }

    if (obfuscationSettings.udp2tcpSettings) {
      const grpcUdp2tcpSettings = new grpcTypes.Udp2TcpObfuscationSettings();
      if (obfuscationSettings.udp2tcpSettings.port !== 'any') {
        grpcUdp2tcpSettings.setPort(obfuscationSettings.udp2tcpSettings.port.only);
      }
      grpcObfuscationSettings.setUdp2tcp(grpcUdp2tcpSettings);
    }

    await this.call<grpcTypes.ObfuscationSettings, Empty>(
      this.client.setObfuscationSettings,
      grpcObfuscationSettings,
    );
  }

  public async setOpenVpnMssfix(mssfix?: number): Promise<void> {
    await this.callNumber(this.client.setOpenvpnMssfix, mssfix);
  }

  public async setWireguardMtu(mtu?: number): Promise<void> {
    await this.callNumber(this.client.setWireguardMtu, mtu);
  }

  public async setWireguardQuantumResistant(quantumResistant?: boolean): Promise<void> {
    const quantumResistantState = new grpcTypes.QuantumResistantState();
    switch (quantumResistant) {
      case true:
        quantumResistantState.setState(grpcTypes.QuantumResistantState.State.ON);
        break;
      case false:
        quantumResistantState.setState(grpcTypes.QuantumResistantState.State.OFF);
        break;
      case undefined:
        quantumResistantState.setState(grpcTypes.QuantumResistantState.State.AUTO);
        break;
    }
    await this.call<grpcTypes.QuantumResistantState, Empty>(
      this.client.setQuantumResistantTunnel,
      quantumResistantState,
    );
  }

  public async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this.callBool(this.client.setAutoConnect, autoConnect);
  }

  public async connectTunnel(): Promise<void> {
    await this.callEmpty(this.client.connectTunnel);
  }

  public async disconnectTunnel(): Promise<void> {
    await this.callEmpty(this.client.disconnectTunnel);
  }

  public async reconnectTunnel(): Promise<void> {
    await this.callEmpty(this.client.reconnectTunnel);
  }

  public async getLocation(): Promise<ILocation> {
    const response = await this.callEmpty<grpcTypes.GeoIpLocation>(this.client.getCurrentLocation);
    return response.toObject();
  }

  public async getState(): Promise<TunnelState> {
    const response = await this.callEmpty<grpcTypes.TunnelState>(this.client.getTunnelState);
    return convertFromTunnelState(response)!;
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.callEmpty<grpcTypes.Settings>(this.client.getSettings);
    return convertFromSettings(response)!;
  }

  public subscribeDaemonEventListener(listener: SubscriptionListener<DaemonEvent>) {
    const call = this.isConnected && this.client.eventsListen(new Empty());
    if (!call) {
      throw noConnectionError;
    }
    const subscriptionId = this.subscriptionId();
    listener.subscriptionId = subscriptionId;
    this.subscriptions.set(subscriptionId, call);

    call.on('data', (data: grpcTypes.DaemonEvent) => {
      try {
        const daemonEvent = convertFromDaemonEvent(data);
        listener.onEvent(daemonEvent);
      } catch (e) {
        const error = e as Error;
        listener.onError(error);
      }
    });

    call.on('error', (error) => {
      listener.onError(error);
      this.removeSubscription(subscriptionId);
    });
  }

  public unsubscribeDaemonEventListener(listener: SubscriptionListener<DaemonEvent>) {
    const id = listener.subscriptionId;
    if (id !== undefined) {
      this.removeSubscription(id);
    }
  }

  public async getAccountHistory(): Promise<AccountToken | undefined> {
    const response = await this.callEmpty<grpcTypes.AccountHistory>(this.client.getAccountHistory);
    return response.getToken()?.getValue();
  }

  public async clearAccountHistory(): Promise<void> {
    await this.callEmpty(this.client.clearAccountHistory);
  }

  public async getCurrentVersion(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client.getCurrentVersion);
    return response.getValue();
  }

  public async setDnsOptions(dns: IDnsOptions): Promise<void> {
    const dnsOptions = new grpcTypes.DnsOptions();

    const defaultOptions = new grpcTypes.DefaultDnsOptions();
    defaultOptions.setBlockAds(dns.defaultOptions.blockAds);
    defaultOptions.setBlockTrackers(dns.defaultOptions.blockTrackers);
    defaultOptions.setBlockMalware(dns.defaultOptions.blockMalware);
    defaultOptions.setBlockAdultContent(dns.defaultOptions.blockAdultContent);
    defaultOptions.setBlockGambling(dns.defaultOptions.blockGambling);
    defaultOptions.setBlockSocialMedia(dns.defaultOptions.blockSocialMedia);
    dnsOptions.setDefaultOptions(defaultOptions);

    const customOptions = new grpcTypes.CustomDnsOptions();
    customOptions.setAddressesList(dns.customOptions.addresses);
    dnsOptions.setCustomOptions(customOptions);

    if (dns.state === 'custom') {
      dnsOptions.setState(grpcTypes.DnsOptions.DnsState.CUSTOM);
    } else {
      dnsOptions.setState(grpcTypes.DnsOptions.DnsState.DEFAULT);
    }

    await this.call<grpcTypes.DnsOptions, Empty>(this.client.setDnsOptions, dnsOptions);
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    const response = await this.callEmpty<grpcTypes.AppVersionInfo>(this.client.getVersionInfo);
    return response.toObject();
  }

  public async addSplitTunnelingApplication(path: string): Promise<void> {
    await this.callString(this.client.addSplitTunnelApp, path);
  }

  public async removeSplitTunnelingApplication(path: string): Promise<void> {
    await this.callString(this.client.removeSplitTunnelApp, path);
  }

  public async setSplitTunnelingState(enabled: boolean): Promise<void> {
    await this.callBool(this.client.setSplitTunnelState, enabled);
  }

  public async checkVolumes(): Promise<void> {
    await this.callEmpty(this.client.checkVolumes);
  }

  public async isPerformingPostUpgrade(): Promise<boolean> {
    const response = await this.callEmpty<BoolValue>(this.client.isPerformingPostUpgrade);
    return response.getValue();
  }

  public async getDevice(): Promise<DeviceState> {
    const response = await this.callEmpty<grpcTypes.DeviceState>(this.client.getDevice);
    return convertFromDeviceState(response);
  }

  public async updateDevice(): Promise<void> {
    await this.callEmpty(this.client.updateDevice);
  }

  public async listDevices(accountToken: AccountToken): Promise<Array<IDevice>> {
    try {
      const response = await this.callString<grpcTypes.DeviceList>(
        this.client.listDevices,
        accountToken,
      );

      return response.getDevicesList().map(convertFromDevice);
    } catch {
      throw new Error('Failed to list devices');
    }
  }

  public async removeDevice(deviceRemoval: IDeviceRemoval): Promise<void> {
    const grpcDeviceRemoval = new grpcTypes.DeviceRemoval();
    grpcDeviceRemoval.setAccountToken(deviceRemoval.accountToken);
    grpcDeviceRemoval.setDeviceId(deviceRemoval.deviceId);

    await this.call<grpcTypes.DeviceRemoval, Empty>(this.client.removeDevice, grpcDeviceRemoval);
  }

  public async createCustomList(name: string): Promise<void | CustomListError> {
    try {
      await this.callString<Empty>(this.client.createCustomList, name);
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code === 6) {
        return { type: 'name already exists' };
      } else {
        throw error;
      }
    }
  }

  public async deleteCustomList(id: string): Promise<void> {
    await this.callString<Empty>(this.client.deleteCustomList, id);
  }

  public async updateCustomList(customList: ICustomList): Promise<void | CustomListError> {
    try {
      await this.call<grpcTypes.CustomList, Empty>(
        this.client.updateCustomList,
        convertToCustomList(customList),
      );
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code === 6) {
        return { type: 'name already exists' };
      } else {
        throw error;
      }
    }
  }

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
    if (fn && this.isConnected) {
      return promisify<T, R>(fn.bind(this.client))(arg);
    } else {
      throw noConnectionError;
    }
  }

  private onClose(error?: Error) {
    const wasConnected = this.isConnectedValue;
    this.isConnectedValue = false;

    this.connectionObserver?.onClose(wasConnected, error);
  }

  private removeSubscription(id: number) {
    const subscription = this.subscriptions.get(id);
    if (subscription !== undefined) {
      this.subscriptions.delete(id);
      subscription.removeAllListeners('data');
      subscription.removeAllListeners('error');

      subscription.on('error', (e) => {
        const error = e as grpc.ServiceError;
        if (error.code !== grpc.status.CANCELLED) {
          throw error;
        }
      });
      // setImmediate is required due to https://github.com/grpc/grpc-node/issues/1464. Should be
      // possible to remove it again after upgrading to Electron 16 which is using a node version
      // where this is fixed.
      setImmediate(() => subscription.cancel());
    }
  }

  private channelOptions(): grpc.ClientOptions {
    /* eslint-disable @typescript-eslint/naming-convention */
    return {
      'grpc.max_reconnect_backoff_ms': 3000,
      'grpc.initial_reconnect_backoff_ms': 3000,
      'grpc.keepalive_time_ms': Math.pow(2, 30),
      'grpc.keepalive_timeout_ms': Math.pow(2, 30),
    };
    /* eslint-enable @typescript-eslint/naming-convention */
  }

  private connectivityChangeCallback(timeoutErr?: Error) {
    const channel = this.client.getChannel();
    const currentState = channel?.getConnectivityState(true);
    log.verbose(`GRPC Channel connectivity state changed to ${currentState}`);
    if (channel) {
      if (timeoutErr) {
        this.setChannelCallback(currentState);
        return;
      }
      const wasConnected = this.isConnected;
      if (this.channelDisconnected(currentState)) {
        this.onClose();
        // Try and reconnect in case
        void this.connect().catch((error) => {
          log.error(`Failed to reconnect - ${error}`);
        });
        this.setChannelCallback(currentState);
      } else if (!wasConnected && currentState === grpc.connectivityState.READY) {
        this.isConnectedValue = true;
        this.connectionObserver?.onOpen();
        this.setChannelCallback(currentState);
      }
    }
  }

  private channelDisconnected(state: grpc.connectivityState): boolean {
    return (
      (state === grpc.connectivityState.SHUTDOWN ||
        state === grpc.connectivityState.TRANSIENT_FAILURE ||
        state === grpc.connectivityState.IDLE) &&
      this.isConnected
    );
  }

  private setChannelCallback(currentState?: grpc.connectivityState) {
    const channel = this.client.getChannel();
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
  private ensureConnectivity() {
    this.reconnectionTimeout = setTimeout(() => {
      const lastState = this.client.getChannel().getConnectivityState(true);
      if (this.channelDisconnected(lastState)) {
        this.onClose();
      }
      if (!this.isConnected) {
        void this.connect().catch((error) => {
          log.error(`Failed to reconnect - ${error}`);
        });
      }
    }, 3000);
  }
}

function liftConstraint<T>(constraint: Constraint<T> | undefined): T | undefined {
  if (constraint !== undefined && constraint !== 'any') {
    return constraint.only;
  }
  return undefined;
}

function convertFromRelayList(relayList: grpcTypes.RelayList): IRelayListWithEndpointData {
  return {
    relayList: {
      countries: relayList
        .getCountriesList()
        .map((country: grpcTypes.RelayListCountry) =>
          convertFromRelayListCountry(country.toObject()),
        ),
    },
    wireguardEndpointData: convertWireguardEndpointData(relayList.getWireguard()!),
  };
}

function convertWireguardEndpointData(
  data: grpcTypes.WireguardEndpointData,
): IWireguardEndpointData {
  return {
    portRanges: data.getPortRangesList().map((range) => [range.getFirst(), range.getLast()]),
    udp2tcpPorts: data.getUdp2tcpPortsList(),
  };
}

function convertFromRelayListCountry(
  country: grpcTypes.RelayListCountry.AsObject,
): IRelayListCountry {
  return {
    ...country,
    cities: country.citiesList.map(convertFromRelayListCity),
  };
}

function convertFromRelayListCity(city: grpcTypes.RelayListCity.AsObject): IRelayListCity {
  return {
    ...city,
    relays: city.relaysList.map(convertFromRelayListRelay),
  };
}

function convertFromRelayListRelay(relay: grpcTypes.Relay.AsObject): IRelayListHostname {
  return {
    ...relay,
    endpointType: convertFromRelayType(relay.endpointType),
  };
}

function convertFromRelayType(relayType: grpcTypes.Relay.RelayType): RelayEndpointType {
  const protocolMap: Record<grpcTypes.Relay.RelayType, RelayEndpointType> = {
    [grpcTypes.Relay.RelayType.OPENVPN]: 'openvpn',
    [grpcTypes.Relay.RelayType.BRIDGE]: 'bridge',
    [grpcTypes.Relay.RelayType.WIREGUARD]: 'wireguard',
  };
  return protocolMap[relayType];
}

function convertFromWireguardKey(publicKey: Uint8Array | string): string {
  if (typeof publicKey === 'string') {
    return publicKey;
  }
  return Buffer.from(publicKey).toString('base64');
}

function convertFromTransportProtocol(protocol: grpcTypes.TransportProtocol): RelayProtocol {
  const protocolMap: Record<grpcTypes.TransportProtocol, RelayProtocol> = {
    [grpcTypes.TransportProtocol.TCP]: 'tcp',
    [grpcTypes.TransportProtocol.UDP]: 'udp',
  };
  return protocolMap[protocol];
}

function convertFromTunnelState(tunnelState: grpcTypes.TunnelState): TunnelState | undefined {
  const tunnelStateObject = tunnelState.toObject();
  switch (tunnelState.getStateCase()) {
    case grpcTypes.TunnelState.StateCase.STATE_NOT_SET:
      return undefined;
    case grpcTypes.TunnelState.StateCase.DISCONNECTED:
      return { state: 'disconnected' };
    case grpcTypes.TunnelState.StateCase.DISCONNECTING: {
      const detailsMap: Record<grpcTypes.AfterDisconnect, AfterDisconnect> = {
        [grpcTypes.AfterDisconnect.NOTHING]: 'nothing',
        [grpcTypes.AfterDisconnect.BLOCK]: 'block',
        [grpcTypes.AfterDisconnect.RECONNECT]: 'reconnect',
      };
      return (
        tunnelStateObject.disconnecting && {
          state: 'disconnecting',
          details: detailsMap[tunnelStateObject.disconnecting.afterDisconnect],
        }
      );
    }
    case grpcTypes.TunnelState.StateCase.ERROR:
      return (
        tunnelStateObject.error?.errorState && {
          state: 'error',
          details: convertFromTunnelStateError(tunnelStateObject.error.errorState),
        }
      );
    case grpcTypes.TunnelState.StateCase.CONNECTING:
      return {
        state: 'connecting',
        details:
          tunnelStateObject.connecting?.relayInfo &&
          convertFromTunnelStateRelayInfo(tunnelStateObject.connecting.relayInfo),
      };
    case grpcTypes.TunnelState.StateCase.CONNECTED: {
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

function convertFromTunnelStateError(state: grpcTypes.ErrorState.AsObject): ErrorState {
  const baseError = {
    blockingError: state.blockingError && convertFromBlockingError(state.blockingError),
  };

  switch (state.cause) {
    case grpcTypes.ErrorState.Cause.AUTH_FAILED:
      return {
        ...baseError,
        cause: ErrorStateCause.authFailed,
        authFailedError: convertFromAuthFailedError(state.authFailedError),
      };
    case grpcTypes.ErrorState.Cause.TUNNEL_PARAMETER_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.tunnelParameterError,
        parameterError: convertFromParameterError(state.parameterError),
      };
    case grpcTypes.ErrorState.Cause.SET_FIREWALL_POLICY_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.setFirewallPolicyError,
        policyError: convertFromBlockingError(state.policyError!),
      };

    case grpcTypes.ErrorState.Cause.IS_OFFLINE:
      return {
        ...baseError,
        cause: ErrorStateCause.isOffline,
      };
    case grpcTypes.ErrorState.Cause.SET_DNS_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.setDnsError,
      };
    case grpcTypes.ErrorState.Cause.IPV6_UNAVAILABLE:
      return {
        ...baseError,
        cause: ErrorStateCause.ipv6Unavailable,
      };
    case grpcTypes.ErrorState.Cause.START_TUNNEL_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.startTunnelError,
      };
    case grpcTypes.ErrorState.Cause.SPLIT_TUNNEL_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.splitTunnelError,
      };
    case grpcTypes.ErrorState.Cause.VPN_PERMISSION_DENIED:
      // VPN_PERMISSION_DENIED is only ever created on Android
      throw invalidErrorStateCause;
  }
}

function convertFromBlockingError(
  error: grpcTypes.ErrorState.FirewallPolicyError.AsObject,
): FirewallPolicyError {
  switch (error.type) {
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.GENERIC:
      return { type: FirewallPolicyErrorType.generic };
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.LOCKED: {
      const pid = error.lockPid;
      const name = error.lockName!;
      return { type: FirewallPolicyErrorType.locked, pid, name };
    }
  }
}

function convertFromAuthFailedError(error: grpcTypes.ErrorState.AuthFailedError): AuthFailedError {
  switch (error) {
    case grpcTypes.ErrorState.AuthFailedError.UNKNOWN:
      return AuthFailedError.unknown;
    case grpcTypes.ErrorState.AuthFailedError.INVALID_ACCOUNT:
      return AuthFailedError.invalidAccount;
    case grpcTypes.ErrorState.AuthFailedError.EXPIRED_ACCOUNT:
      return AuthFailedError.expiredAccount;
    case grpcTypes.ErrorState.AuthFailedError.TOO_MANY_CONNECTIONS:
      return AuthFailedError.tooManyConnections;
  }
}

function convertFromParameterError(
  error: grpcTypes.ErrorState.GenerationError,
): TunnelParameterError {
  switch (error) {
    case grpcTypes.ErrorState.GenerationError.NO_MATCHING_RELAY:
      return TunnelParameterError.noMatchingRelay;
    case grpcTypes.ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY:
      return TunnelParameterError.noMatchingBridgeRelay;
    case grpcTypes.ErrorState.GenerationError.NO_WIREGUARD_KEY:
      return TunnelParameterError.noWireguardKey;
    case grpcTypes.ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR:
      return TunnelParameterError.customTunnelHostResolutionError;
  }
}

function convertFromTunnelStateRelayInfo(
  state: grpcTypes.TunnelStateRelayInfo.AsObject,
): ITunnelStateRelayInfo | undefined {
  if (state.tunnelEndpoint) {
    return {
      ...state,
      endpoint: {
        ...state.tunnelEndpoint,
        tunnelType: convertFromTunnelType(state.tunnelEndpoint.tunnelType),
        protocol: convertFromTransportProtocol(state.tunnelEndpoint.protocol),
        proxy: state.tunnelEndpoint.proxy && convertFromProxyEndpoint(state.tunnelEndpoint.proxy),
        obfuscationEndpoint:
          state.tunnelEndpoint.obfuscation &&
          convertFromObfuscationEndpoint(state.tunnelEndpoint.obfuscation),
        entryEndpoint:
          state.tunnelEndpoint.entryEndpoint &&
          convertFromEntryEndpoint(state.tunnelEndpoint.entryEndpoint),
      },
    };
  }
  return undefined;
}

function convertFromTunnelType(tunnelType: grpcTypes.TunnelType): TunnelType {
  const tunnelTypeMap: Record<grpcTypes.TunnelType, TunnelType> = {
    [grpcTypes.TunnelType.WIREGUARD]: 'wireguard',
    [grpcTypes.TunnelType.OPENVPN]: 'openvpn',
  };

  return tunnelTypeMap[tunnelType];
}

function convertFromProxyEndpoint(proxyEndpoint: grpcTypes.ProxyEndpoint.AsObject): IProxyEndpoint {
  const proxyTypeMap: Record<grpcTypes.ProxyType, ProxyType> = {
    [grpcTypes.ProxyType.CUSTOM]: 'custom',
    [grpcTypes.ProxyType.SHADOWSOCKS]: 'shadowsocks',
  };

  return {
    ...proxyEndpoint,
    protocol: convertFromTransportProtocol(proxyEndpoint.protocol),
    proxyType: proxyTypeMap[proxyEndpoint.proxyType],
  };
}

function convertFromObfuscationEndpoint(
  obfuscationEndpoint: grpcTypes.ObfuscationEndpoint.AsObject,
): IObfuscationEndpoint {
  const obfuscationTypes: Record<grpcTypes.ObfuscationType, EndpointObfuscationType> = {
    [grpcTypes.ObfuscationType.UDP2TCP]: 'udp2tcp',
  };

  return {
    ...obfuscationEndpoint,
    protocol: convertFromTransportProtocol(obfuscationEndpoint.protocol),
    obfuscationType: obfuscationTypes[obfuscationEndpoint.obfuscationType],
  };
}

function convertFromEntryEndpoint(entryEndpoint: grpcTypes.Endpoint.AsObject) {
  return {
    address: entryEndpoint.address,
    transportProtocol: convertFromTransportProtocol(entryEndpoint.protocol),
  };
}

function convertFromSettings(settings: grpcTypes.Settings): ISettings | undefined {
  const settingsObject = settings.toObject();
  const bridgeState = convertFromBridgeState(settingsObject.bridgeState!.state!);
  const relaySettings = convertFromRelaySettings(settings.getRelaySettings())!;
  const bridgeSettings = convertFromBridgeSettings(settings.getBridgeSettings()!);
  const tunnelOptions = convertFromTunnelOptions(settingsObject.tunnelOptions!);
  const splitTunnel = settingsObject.splitTunnel ?? { enableExclusions: false, appsList: [] };
  const obfuscationSettings = convertFromObfuscationSettings(settingsObject.obfuscationSettings);
  const customLists = convertFromCustomListSettings(settings.getCustomLists());
  return {
    ...settings.toObject(),
    bridgeState,
    relaySettings,
    bridgeSettings,
    tunnelOptions,
    splitTunnel,
    obfuscationSettings,
    customLists,
  };
}

function convertFromBridgeState(bridgeState: grpcTypes.BridgeState.State): BridgeState {
  const bridgeStateMap: Record<grpcTypes.BridgeState.State, BridgeState> = {
    [grpcTypes.BridgeState.State.AUTO]: 'auto',
    [grpcTypes.BridgeState.State.ON]: 'on',
    [grpcTypes.BridgeState.State.OFF]: 'off',
  };

  return bridgeStateMap[bridgeState];
}

function convertFromRelaySettings(
  relaySettings?: grpcTypes.RelaySettings,
): RelaySettings | undefined {
  if (relaySettings) {
    switch (relaySettings.getEndpointCase()) {
      case grpcTypes.RelaySettings.EndpointCase.ENDPOINT_NOT_SET:
        return undefined;
      case grpcTypes.RelaySettings.EndpointCase.CUSTOM: {
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
      case grpcTypes.RelaySettings.EndpointCase.NORMAL: {
        const normal = relaySettings.getNormal()!;
        const locationConstraint = convertFromLocationConstraint(normal.getLocation());
        const location = wrapConstraint(locationConstraint);
        // `getTunnelType()` is not falsy if type is 'any'
        const tunnelProtocol = convertFromTunnelTypeConstraint(
          normal.hasTunnelType() ? normal.getTunnelType() : undefined,
        );
        const providers = normal.getProvidersList();
        const ownership = convertFromOwnership(normal.getOwnership());
        const openvpnConstraints = convertFromOpenVpnConstraints(normal.getOpenvpnConstraints()!);
        const wireguardConstraints = convertFromWireguardConstraints(
          normal.getWireguardConstraints()!,
        );

        return {
          normal: {
            location,
            tunnelProtocol,
            providers,
            ownership,
            wireguardConstraints,
            openvpnConstraints,
          },
        };
      }
    }
  } else {
    return undefined;
  }
}

function convertFromBridgeSettings(bridgeSettings: grpcTypes.BridgeSettings): BridgeSettings {
  const bridgeSettingsObject = bridgeSettings.toObject();
  const normalSettings = bridgeSettingsObject.normal;
  if (normalSettings) {
    const locationConstraint = convertFromLocationConstraint(
      bridgeSettings.getNormal()?.getLocation(),
    );
    const location = wrapConstraint(locationConstraint);
    const providers = normalSettings.providersList;
    const ownership = convertFromOwnership(normalSettings.ownership);
    return {
      normal: {
        location,
        providers,
        ownership,
      },
    };
  }

  const customSettings = (settings: ProxySettings): BridgeSettings => {
    return { custom: settings };
  };

  const localSettings = bridgeSettingsObject.local;
  if (localSettings) {
    return customSettings({
      port: localSettings.port,
      peer: localSettings.peer,
    });
  }

  const remoteSettings = bridgeSettingsObject.remote;
  if (remoteSettings) {
    return customSettings({
      address: remoteSettings.address,
      auth: remoteSettings.auth && { ...remoteSettings.auth },
    });
  }

  const shadowsocksSettings = bridgeSettingsObject.shadowsocks!;
  return customSettings({
    peer: shadowsocksSettings.peer!,
    password: shadowsocksSettings.password!,
    cipher: shadowsocksSettings.cipher!,
  });
}

function convertFromConnectionConfig(
  connectionConfig: grpcTypes.ConnectionConfig,
): ConnectionConfig | undefined {
  const connectionConfigObject = connectionConfig.toObject();
  switch (connectionConfig.getConfigCase()) {
    case grpcTypes.ConnectionConfig.ConfigCase.CONFIG_NOT_SET:
      return undefined;
    case grpcTypes.ConnectionConfig.ConfigCase.WIREGUARD:
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
    case grpcTypes.ConnectionConfig.ConfigCase.OPENVPN: {
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
}

function convertFromLocationConstraint(
  location?: grpcTypes.LocationConstraint,
): RelayLocation | undefined {
  if (location === undefined) {
    return undefined;
  } else if (location.getTypeCase() === grpcTypes.LocationConstraint.TypeCase.CUSTOM_LIST) {
    return { customList: location.getCustomList() };
  } else {
    const innerLocation = location.getLocation()?.toObject();
    return innerLocation && convertFromGeographicConstraint(innerLocation);
  }
}

function convertFromGeographicConstraint(
  location: grpcTypes.GeographicLocationConstraint.AsObject,
): RelayLocation {
  if (location.hostname) {
    return location;
  } else if (location.city) {
    return {
      country: location.country,
      city: location.city,
    };
  } else {
    return {
      country: location.country,
    };
  }
}

function convertFromTunnelOptions(tunnelOptions: grpcTypes.TunnelOptions.AsObject): ITunnelOptions {
  return {
    openvpn: {
      mssfix: tunnelOptions.openvpn!.mssfix,
    },
    wireguard: {
      mtu: tunnelOptions.wireguard!.mtu,
      quantumResistant: convertFromQuantumResistantState(
        tunnelOptions.wireguard?.quantumResistant?.state,
      ),
    },
    generic: {
      enableIpv6: tunnelOptions.generic!.enableIpv6,
    },
    dns: {
      state:
        tunnelOptions.dnsOptions?.state === grpcTypes.DnsOptions.DnsState.CUSTOM
          ? 'custom'
          : 'default',
      defaultOptions: {
        blockAds: tunnelOptions.dnsOptions?.defaultOptions?.blockAds ?? false,
        blockTrackers: tunnelOptions.dnsOptions?.defaultOptions?.blockTrackers ?? false,
        blockMalware: tunnelOptions.dnsOptions?.defaultOptions?.blockMalware ?? false,
        blockAdultContent: tunnelOptions.dnsOptions?.defaultOptions?.blockAdultContent ?? false,
        blockGambling: tunnelOptions.dnsOptions?.defaultOptions?.blockGambling ?? false,
        blockSocialMedia: tunnelOptions.dnsOptions?.defaultOptions?.blockSocialMedia ?? false,
      },
      customOptions: {
        addresses: tunnelOptions.dnsOptions?.customOptions?.addressesList ?? [],
      },
    },
  };
}

function convertFromQuantumResistantState(
  state?: grpcTypes.QuantumResistantState.State,
): boolean | undefined {
  return state === undefined
    ? undefined
    : {
        [grpcTypes.QuantumResistantState.State.ON]: true,
        [grpcTypes.QuantumResistantState.State.OFF]: false,
        [grpcTypes.QuantumResistantState.State.AUTO]: undefined,
      }[state];
}

function convertFromObfuscationSettings(
  obfuscationSettings?: grpcTypes.ObfuscationSettings.AsObject,
): ObfuscationSettings {
  let selectedObfuscationType = ObfuscationType.auto;
  switch (obfuscationSettings?.selectedObfuscation) {
    case grpcTypes.ObfuscationSettings.SelectedObfuscation.OFF:
      selectedObfuscationType = ObfuscationType.off;
      break;
    case grpcTypes.ObfuscationSettings.SelectedObfuscation.UDP2TCP:
      selectedObfuscationType = ObfuscationType.udp2tcp;
      break;
  }

  return {
    selectedObfuscation: selectedObfuscationType,
    udp2tcpSettings: obfuscationSettings?.udp2tcp
      ? { port: convertFromConstraint(obfuscationSettings.udp2tcp.port) }
      : { port: 'any' },
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
    return { relayList: convertFromRelayList(relayList) };
  }

  const deviceConfig = data.getDevice();
  if (deviceConfig !== undefined) {
    return { device: convertFromDeviceEvent(deviceConfig) };
  }

  const deviceRemoval = data.getRemoveDevice();
  if (deviceRemoval !== undefined) {
    return { deviceRemoval: convertFromDeviceRemoval(deviceRemoval) };
  }

  const versionInfo = data.getVersionInfo();
  if (versionInfo !== undefined) {
    return { appVersionInfo: versionInfo.toObject() };
  }

  // Handle unknown daemon events
  const keys = Object.entries(data.toObject())
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
  throw new Error(`Unknown daemon event received containing ${keys}`);
}

function convertFromOwnership(ownership: grpcTypes.Ownership): Ownership {
  switch (ownership) {
    case grpcTypes.Ownership.ANY:
      return Ownership.any;
    case grpcTypes.Ownership.MULLVAD_OWNED:
      return Ownership.mullvadOwned;
    case grpcTypes.Ownership.RENTED:
      return Ownership.rented;
  }
}

function convertToOwnership(ownership: Ownership): grpcTypes.Ownership {
  switch (ownership) {
    case Ownership.any:
      return grpcTypes.Ownership.ANY;
    case Ownership.mullvadOwned:
      return grpcTypes.Ownership.MULLVAD_OWNED;
    case Ownership.rented:
      return grpcTypes.Ownership.RENTED;
  }
}

function convertFromOpenVpnConstraints(
  constraints: grpcTypes.OpenvpnConstraints,
): IOpenVpnConstraints {
  const transportPort = convertFromConstraint(constraints.getPort());
  if (transportPort !== 'any' && 'only' in transportPort) {
    const port = convertFromConstraint(transportPort.only.getPort());
    let protocol: Constraint<RelayProtocol> = 'any';
    switch (transportPort.only.getProtocol()) {
      case grpcTypes.TransportProtocol.TCP:
        protocol = { only: 'tcp' };
        break;
      case grpcTypes.TransportProtocol.UDP:
        protocol = { only: 'udp' };
        break;
    }
    return { port, protocol };
  }
  return { port: 'any', protocol: 'any' };
}

function convertFromWireguardConstraints(
  constraints: grpcTypes.WireguardConstraints,
): IWireguardConstraints {
  const result: IWireguardConstraints = {
    port: 'any',
    ipVersion: 'any',
    useMultihop: constraints.getUseMultihop(),
    entryLocation: 'any',
  };

  const port = constraints.getPort();
  if (port) {
    result.port = { only: port };
  }

  // `getIpVersion()` is not falsy if type is 'any'
  if (constraints.hasIpVersion()) {
    switch (constraints.getIpVersion()) {
      case grpcTypes.IpVersion.V4:
        result.ipVersion = { only: 'ipv4' };
        break;
      case grpcTypes.IpVersion.V6:
        result.ipVersion = { only: 'ipv6' };
        break;
    }
  }

  const entryLocation = constraints.getEntryLocation();
  if (entryLocation) {
    const location = convertFromLocationConstraint(entryLocation);
    result.entryLocation = wrapConstraint(location);
  }

  return result;
}

function convertFromTunnelTypeConstraint(
  constraint: grpcTypes.TunnelType | undefined,
): Constraint<TunnelProtocol> {
  switch (constraint) {
    case grpcTypes.TunnelType.WIREGUARD: {
      return { only: 'wireguard' };
    }
    case grpcTypes.TunnelType.OPENVPN: {
      return { only: 'openvpn' };
    }
    default: {
      return 'any';
    }
  }
}

function convertFromConstraint<T>(value: T | undefined): Constraint<T> {
  if (value) {
    return { only: value };
  } else {
    return 'any';
  }
}

function convertToRelayConstraints(
  constraints: IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints>,
): grpcTypes.NormalRelaySettings {
  const relayConstraints = new grpcTypes.NormalRelaySettings();

  if (constraints.tunnelProtocol !== 'any') {
    relayConstraints.setTunnelType(convertToTunnelType(constraints.tunnelProtocol.only));
  }
  relayConstraints.setLocation(convertToLocation(liftConstraint(constraints.location)));
  relayConstraints.setWireguardConstraints(
    convertToWireguardConstraints(constraints.wireguardConstraints),
  );
  relayConstraints.setOpenvpnConstraints(
    convertToOpenVpnConstraints(constraints.openvpnConstraints),
  );
  relayConstraints.setProvidersList(constraints.providers);
  relayConstraints.setOwnership(convertToOwnership(constraints.ownership));

  return relayConstraints;
}

function convertToNormalBridgeSettings(
  constraints: IBridgeConstraints,
): grpcTypes.BridgeSettings.BridgeConstraints {
  const normalBridgeSettings = new grpcTypes.BridgeSettings.BridgeConstraints();
  normalBridgeSettings.setLocation(convertToLocation(liftConstraint(constraints.location)));
  normalBridgeSettings.setProvidersList(constraints.providers);

  return normalBridgeSettings;
}

function convertToLocation(
  constraint: RelayLocation | undefined,
): grpcTypes.LocationConstraint | undefined {
  const locationConstraint = new grpcTypes.LocationConstraint();
  if (constraint && 'customList' in constraint && constraint.customList) {
    locationConstraint.setCustomList(constraint.customList);
  } else {
    const location = constraint && convertToGeographicConstraint(constraint);
    locationConstraint.setLocation(location);
  }

  return locationConstraint;
}

function convertToGeographicConstraint(
  location: RelayLocation,
): grpcTypes.GeographicLocationConstraint {
  const relayLocation = new grpcTypes.GeographicLocationConstraint();
  if ('hostname' in location) {
    relayLocation.setCountry(location.country);
    relayLocation.setCity(location.city);
    relayLocation.setHostname(location.hostname);
  } else if ('city' in location) {
    relayLocation.setCountry(location.country);
    relayLocation.setCity(location.city);
  } else if ('country' in location) {
    relayLocation.setCountry(location.country);
  }

  return relayLocation;
}

function convertToTunnelType(tunnelProtocol: TunnelProtocol): grpcTypes.TunnelType {
  switch (tunnelProtocol) {
    case 'wireguard':
      return grpcTypes.TunnelType.WIREGUARD;
    case 'openvpn':
      return grpcTypes.TunnelType.OPENVPN;
  }
}

function convertToOpenVpnConstraints(
  constraints: Partial<IOpenVpnConstraints> | undefined,
): grpcTypes.OpenvpnConstraints | undefined {
  const openvpnConstraints = new grpcTypes.OpenvpnConstraints();
  if (constraints) {
    const protocol = liftConstraint(constraints.protocol);
    if (protocol) {
      const portConstraints = new grpcTypes.TransportPort();
      const port = liftConstraint(constraints.port);
      if (port) {
        portConstraints.setPort(port);
      }
      portConstraints.setProtocol(convertToTransportProtocol(protocol));
      openvpnConstraints.setPort(portConstraints);
    }
    return openvpnConstraints;
  }

  return undefined;
}

function convertToWireguardConstraints(
  constraint: Partial<IWireguardConstraints> | undefined,
): grpcTypes.WireguardConstraints | undefined {
  if (constraint) {
    const wireguardConstraints = new grpcTypes.WireguardConstraints();

    const port = liftConstraint(constraint.port);
    if (port) {
      wireguardConstraints.setPort(port);
    }

    const ipVersion = liftConstraint(constraint.ipVersion);
    if (ipVersion) {
      const ipVersionProtocol =
        ipVersion === 'ipv4' ? grpcTypes.IpVersion.V4 : grpcTypes.IpVersion.V6;
      wireguardConstraints.setIpVersion(ipVersionProtocol);
    }

    if (constraint.useMultihop) {
      wireguardConstraints.setUseMultihop(constraint.useMultihop);
    }

    const entryLocation = liftConstraint(constraint.entryLocation);
    if (entryLocation) {
      const entryLocationConstraint = convertToLocation(entryLocation);
      wireguardConstraints.setEntryLocation(entryLocationConstraint);
    }

    return wireguardConstraints;
  }
  return undefined;
}

function convertToTransportProtocol(protocol: RelayProtocol): grpcTypes.TransportProtocol {
  switch (protocol) {
    case 'udp':
      return grpcTypes.TransportProtocol.UDP;
    case 'tcp':
      return grpcTypes.TransportProtocol.TCP;
  }
}

function convertFromDeviceEvent(deviceEvent: grpcTypes.DeviceEvent): DeviceEvent {
  const deviceState = convertFromDeviceState(deviceEvent.getNewState()!);
  switch (deviceEvent.getCause()) {
    case grpcTypes.DeviceEvent.Cause.LOGGED_IN:
      return { type: 'logged in', deviceState: deviceState as LoggedInDeviceState };
    case grpcTypes.DeviceEvent.Cause.LOGGED_OUT:
      return { type: 'logged out', deviceState: deviceState as LoggedOutDeviceState };
    case grpcTypes.DeviceEvent.Cause.REVOKED:
      return { type: 'revoked', deviceState: deviceState as LoggedOutDeviceState };
    case grpcTypes.DeviceEvent.Cause.UPDATED:
      return { type: 'updated', deviceState: deviceState as LoggedInDeviceState };
    case grpcTypes.DeviceEvent.Cause.ROTATED_KEY:
      return { type: 'rotated_key', deviceState: deviceState as LoggedInDeviceState };
  }
}

function convertFromDeviceState(deviceState: grpcTypes.DeviceState): DeviceState {
  switch (deviceState.getState()) {
    case grpcTypes.DeviceState.State.LOGGED_IN: {
      const accountAndDevice = deviceState.getDevice()!;
      const device = accountAndDevice.getDevice();
      return {
        type: 'logged in',
        accountAndDevice: {
          accountToken: accountAndDevice.getAccountToken(),
          device: device && convertFromDevice(device),
        },
      };
    }
    case grpcTypes.DeviceState.State.LOGGED_OUT:
      return { type: 'logged out' };
    case grpcTypes.DeviceState.State.REVOKED:
      return { type: 'revoked' };
  }
}

function convertFromDeviceRemoval(deviceRemoval: grpcTypes.RemoveDeviceEvent): Array<IDevice> {
  return deviceRemoval.getNewDeviceListList().map(convertFromDevice);
}

function convertFromDevice(device: grpcTypes.Device): IDevice {
  const created = ensureExists(device.getCreated(), "no 'created' field for device").toDate();
  const asObject = device.toObject();

  return {
    ...asObject,
    created: created,
  };
}

function convertFromCustomListSettings(
  customListSettings?: grpcTypes.CustomListSettings,
): CustomLists {
  return customListSettings ? convertFromCustomLists(customListSettings.getCustomListsList()) : [];
}

function convertFromCustomLists(customLists: Array<grpcTypes.CustomList>): CustomLists {
  return customLists.map((list) => ({
    id: list.getId(),
    name: list.getName(),
    locations: list
      .getLocationsList()
      .map((location) =>
        convertFromGeographicConstraint(location.toObject()),
      ) as Array<RelayLocationGeographical>,
  }));
}

function convertToCustomList(customList: ICustomList): grpcTypes.CustomList {
  const grpcCustomList = new grpcTypes.CustomList();
  grpcCustomList.setId(customList.id);
  grpcCustomList.setName(customList.name);

  const locations = customList.locations.map(convertToGeographicConstraint);
  grpcCustomList.setLocationsList(locations);

  return grpcCustomList;
}

function ensureExists<T>(value: T | undefined, errorMessage: string): T {
  if (value) {
    return value;
  }
  throw new ResponseParseError(errorMessage);
}
