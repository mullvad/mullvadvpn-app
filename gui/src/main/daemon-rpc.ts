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
  BridgeSettings,
  BridgeState,
  CustomListError,
  DaemonEvent,
  DeviceState,
  IAppVersionInfo,
  ICustomList,
  IDevice,
  IDeviceRemoval,
  IDnsOptions,
  ILocation,
  IRelayListWithEndpointData,
  ISettings,
  ObfuscationSettings,
  ObfuscationType,
  RelaySettings,
  TunnelState,
  VoucherResponse,
} from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import {
  convertFromDaemonEvent,
  convertFromDevice,
  convertFromDeviceState,
  convertFromRelayList,
  convertFromSettings,
  convertFromTunnelState,
  convertToCustomList,
  convertToNormalBridgeSettings,
  convertToRelayConstraints,
  ensureExists,
} from './grpc-conversion';
import { ManagementServiceClient } from './management_interface/management_interface_grpc_pb';
import * as grpcTypes from './management_interface/management_interface_pb';

const DAEMON_RPC_PATH =
  process.platform === 'win32' ? 'unix:////./pipe/Mullvad VPN' : 'unix:///var/run/mullvad-vpn';

const NETWORK_CALL_TIMEOUT = 10000;
const CHANNEL_STATE_TIMEOUT = 1000 * 60 * 60;

const noConnectionError = new Error('No connection established to daemon');
const configNotSupported = new Error('Setting custom settings is not supported');

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

type CallFunctionArgument<T, R> =
  | ((arg: T, callback: (error: Error | null, result: R) => void) => void)
  | undefined;

class GrpcClient {
  protected client: ManagementServiceClient;
  private isConnectedValue = false;
  private isClosed = false;
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

    this.isClosed = true;
    this.client.close();
    this.connectionObserver = undefined;
    if (this.reconnectionTimeout) {
      clearTimeout(this.reconnectionTimeout);
    }
  }

  protected callEmpty<R>(fn: CallFunctionArgument<Empty, R>): Promise<R> {
    return this.call<Empty, R>(fn, new Empty());
  }

  protected callString<R>(fn: CallFunctionArgument<StringValue, R>, value?: string): Promise<R> {
    const googleString = new StringValue();

    if (value !== undefined) {
      googleString.setValue(value);
    }

    return this.call<StringValue, R>(fn, googleString);
  }

  protected callBool<R>(fn: CallFunctionArgument<BoolValue, R>, value?: boolean): Promise<R> {
    const googleBool = new BoolValue();

    if (value !== undefined) {
      googleBool.setValue(value);
    }

    return this.call<BoolValue, R>(fn, googleBool);
  }

  protected callNumber<R>(fn: CallFunctionArgument<UInt32Value, R>, value?: number): Promise<R> {
    const googleNumber = new UInt32Value();

    if (value !== undefined) {
      googleNumber.setValue(value);
    }

    return this.call<UInt32Value, R>(fn, googleNumber);
  }

  protected call<T, R>(fn: CallFunctionArgument<T, R>, arg: T): Promise<R> {
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

  private deadlineFromNow() {
    return Date.now() + NETWORK_CALL_TIMEOUT;
  }

  private channelStateTimeout(): number {
    return Date.now() + CHANNEL_STATE_TIMEOUT;
  }
}

export class DaemonRpc extends GrpcClient {
  private nextSubscriptionId = 0;
  private subscriptions: Map<number, grpc.ClientReadableStream<grpcTypes.DaemonEvent>> = new Map();

  public disconnect() {
    for (const subscriptionId of this.subscriptions.keys()) {
      this.removeSubscription(subscriptionId);
    }

    super.disconnect();
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
  public async setRelaySettings(relaySettings: RelaySettings): Promise<void> {
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
}
