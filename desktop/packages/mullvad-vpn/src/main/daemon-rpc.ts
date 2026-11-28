import * as grpc from '@grpc/grpc-js';
import { Empty } from 'google-protobuf/google/protobuf/empty_pb.js';
import { BoolValue, StringValue } from 'google-protobuf/google/protobuf/wrappers_pb.js';
import { ManagementServiceClient } from 'management-interface/management-interface';
import * as grpcTypes from 'management-interface/management-interface/grpc-types';

import {
  AccessMethodExistsError,
  AccessMethodSetting,
  CustomListError,
  CustomProxy,
  DaemonAppUpgradeEvent,
  DaemonEvent,
  DisconnectSource,
  IAppVersionInfo,
  ICustomList,
  IDnsOptions,
  IRelayListWithEndpointData,
  ISettings,
  IWireguardIdentity,
  NewAccessMethodSetting,
  NewCustomList,
  ObfuscationSettings,
  RelaySettings,
  type SettingsMigration,
  type ShadowsocksCipher,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { ConnectionObserver, GrpcClient, noConnectionError } from './grpc-client';
import {
  convertFromApiAccessMethodSetting,
  convertFromAppUpgradeEvent,
  convertFromAppVersionInfo,
  convertFromDaemonEvent,
  convertFromGrpcShadowsocksCiphers,
  convertFromMigrationEvent,
  convertFromRelayList,
  convertFromSettings,
  convertFromTunnelState,
  convertFromWireguardIdentity,
  convertFromWireguardIdentityState,
  convertToAntiCensorshipSettings,
  convertToApiAccessMethodSetting,
  convertToCustomList,
  convertToCustomProxy,
  convertToNewApiAccessMethodSetting,
  convertToNewCustomList,
  convertToRelayConstraints,
} from './grpc-type-convertions';

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

export class DaemonRpc extends GrpcClient<ManagementServiceClient> {
  private nextSubscriptionId = 0;
  private subscriptions: Map<
    number,
    grpc.ClientReadableStream<grpcTypes.DaemonEvent | grpcTypes.AppUpgradeEvent>
  > = new Map();

  public constructor(connectionObserver?: ConnectionObserver) {
    super(connectionObserver);
  }

  createClient(): ManagementServiceClient {
    return new ManagementServiceClient(
      this.prefixedRpcPath(),
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    );
  }

  public disconnect() {
    for (const subscriptionId of this.subscriptions.keys()) {
      this.removeSubscription(subscriptionId);
    }

    super.disconnect();
  }

  public subscribeAppUpgradeEventListener(listener: SubscriptionListener<DaemonAppUpgradeEvent>) {
    const call = this.isConnected && this.client.appUpgradeEventsListen(new Empty());
    if (!call) {
      throw noConnectionError;
    }
    const subscriptionId = this.subscriptionId();
    listener.subscriptionId = subscriptionId;
    this.subscriptions.set(subscriptionId, call);

    call.on('data', (data: grpcTypes.AppUpgradeEvent) => {
      try {
        const appUpgradeEvent = convertFromAppUpgradeEvent(data);
        listener.onEvent(appUpgradeEvent);
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

  public appUpgrade() {
    void this.callEmpty(this.client.appUpgrade);
  }

  public appUpgradeAbort() {
    void this.callEmpty(this.client.appUpgradeAbort);
  }

  public async getAppUpgradeCacheDir(): Promise<string> {
    const response = await this.callEmpty<StringValue>(this.client.getAppUpgradeCacheDir);
    return response.getValue();
  }

  public unsubscribeAppUpgradeEventListener(listener: SubscriptionListener<DaemonAppUpgradeEvent>) {
    const id = listener.subscriptionId;
    if (id !== undefined) {
      this.removeSubscription(id);
    }
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

  public async getRelayLocations(): Promise<IRelayListWithEndpointData> {
    if (this.isConnected) {
      const response = await this.callEmpty<grpcTypes.RelayList>(this.client.getRelayLocations);
      return convertFromRelayList(response);
    } else {
      throw noConnectionError;
    }
  }

  public async importWireguardConfig(config: string): Promise<IWireguardIdentity> {
    const response = await this.callString<grpcTypes.WireguardIdentity>(
      this.client.importWireguardConfig,
      config,
    );
    return convertFromWireguardIdentity(response);
  }

  public async clearWireguardIdentity(): Promise<void> {
    await this.callEmpty(this.client.clearWireguardIdentity);
  }

  public async getWireguardIdentity(): Promise<IWireguardIdentity | undefined> {
    const response = await this.callEmpty<grpcTypes.WireguardIdentityState>(
      this.client.getWireguardIdentity,
    );
    return convertFromWireguardIdentityState(response);
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

  public async setLockdownMode(lockdownMode: boolean): Promise<void> {
    await this.callBool(this.client.setLockdownMode, lockdownMode);
  }

  public async setObfuscationSettings(obfuscationSettings: ObfuscationSettings): Promise<void> {
    const grpcObfuscationSettings = convertToAntiCensorshipSettings(obfuscationSettings);

    await this.call<grpcTypes.ObfuscationSettings, Empty>(
      this.client.setObfuscationSettings,
      grpcObfuscationSettings,
    );
  }

  public async setWireguardMtu(mtu?: number): Promise<void> {
    await this.callNumber(this.client.setWireguardMtu, mtu);
  }

  public async setWireguardQuantumResistant(quantumResistant: boolean): Promise<void> {
    const quantumResistantState = new grpcTypes.QuantumResistantState();
    switch (quantumResistant) {
      case true:
        quantumResistantState.setState(grpcTypes.QuantumResistantState.State.ON);
        break;
      case false:
        quantumResistantState.setState(grpcTypes.QuantumResistantState.State.OFF);
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

  public async disconnectTunnel(source: DisconnectSource): Promise<void> {
    const prefixedSource = `desktop ${source}`;
    await this.callString(this.client.disconnectTunnel, prefixedSource);
  }

  public async reconnectTunnel(): Promise<void> {
    await this.callEmpty(this.client.reconnectTunnel);
  }

  public async getState(): Promise<TunnelState> {
    const response = await this.callEmpty<grpcTypes.TunnelState>(this.client.getTunnelState);
    return convertFromTunnelState(response)!;
  }

  public async getSettings(): Promise<ISettings> {
    const response = await this.callEmpty<grpcTypes.Settings>(this.client.getSettings);
    return convertFromSettings(response)!;
  }

  public async getSettingsMigrations(): Promise<SettingsMigration[]> {
    const response = await this.callEmpty<grpcTypes.SplitFilterMigration>(
      this.client.getMigrationEvent,
    );

    const migrations = convertFromMigrationEvent(response);

    // The `one-a` split filter migration should not be displayed to the user.
    // Remove the 'one-a' split filter migration from the list and return rest.
    const filteredMigrations = migrations.filter(
      (migration) => !(migration.type === 'split-filter' && migration.scenario === 'one-a'),
    );

    return filteredMigrations;
  }

  public async clearSettingsMigrations(): Promise<void> {
    await this.callEmpty(this.client.clearMigrationMessage);
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
    const versionInfo = convertFromAppVersionInfo(response);

    return versionInfo;
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

  public async splitTunnelIsSupported(): Promise<boolean> {
    try {
      const isSupported = await this.callEmpty<BoolValue>(this.client.splitTunnelIsSupported);
      return isSupported.getValue();
    } catch {
      return false;
    }
  }

  public async needFullDiskPermissions(): Promise<boolean> {
    const needFullDiskPermissions = await this.callEmpty<BoolValue>(
      this.client.needFullDiskPermissions,
    );
    return needFullDiskPermissions.getValue();
  }

  public async checkVolumes(): Promise<void> {
    await this.callEmpty(this.client.checkVolumes);
  }

  public async isPerformingPostUpgrade(): Promise<boolean> {
    const response = await this.callEmpty<BoolValue>(this.client.isPerformingPostUpgrade);
    return response.getValue();
  }

  public async prepareRestart(quit: boolean) {
    await this.callBool(this.client.prepareRestartV2, quit);
  }

  public async setEnableDaita(value: boolean): Promise<void> {
    await this.callBool(this.client.setEnableDaita, value);
  }

  public async createCustomList(newCustomList: NewCustomList): Promise<void | CustomListError> {
    try {
      await this.call<grpcTypes.NewCustomList, StringValue>(
        this.client.createCustomList,
        convertToNewCustomList(newCustomList),
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

  public async addApiAccessMethod(
    method: NewAccessMethodSetting,
  ): Promise<string | AccessMethodExistsError> {
    try {
      const result = await this.call<grpcTypes.NewAccessMethodSetting, grpcTypes.UUID>(
        this.client.addApiAccessMethod,
        convertToNewApiAccessMethodSetting(method),
      );
      return result.getValue();
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code === 6) {
        return { type: 'name already exists' };
      } else {
        throw error;
      }
    }
  }

  public async updateApiAccessMethod(
    method: AccessMethodSetting,
  ): Promise<void | AccessMethodExistsError> {
    try {
      await this.call(this.client.updateApiAccessMethod, convertToApiAccessMethodSetting(method));
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code === 6) {
        return { type: 'name already exists' };
      } else {
        throw error;
      }
    }
  }

  public async getCurrentApiAccessMethod() {
    const response = await this.callEmpty<grpcTypes.AccessMethodSetting>(
      this.client.getCurrentApiAccessMethod,
    );
    return convertFromApiAccessMethodSetting(response);
  }

  public async getShadowsocksCiphers(): Promise<ShadowsocksCipher[]> {
    const shadowsocksCiphers = await this.callEmpty<grpcTypes.Shadowsocks.Ciphers>(
      this.client.shadowsocksCiphers,
    );
    const shadowsocksCiphersList = shadowsocksCiphers.getCiphersList();
    return convertFromGrpcShadowsocksCiphers(shadowsocksCiphersList);
  }

  public async removeApiAccessMethod(id: string) {
    const uuid = new grpcTypes.UUID();
    uuid.setValue(id);
    await this.call(this.client.removeApiAccessMethod, uuid);
  }

  public async setApiAccessMethod(id: string) {
    const uuid = new grpcTypes.UUID();
    uuid.setValue(id);
    await this.call(this.client.setApiAccessMethod, uuid);
  }

  public async testApiAccessMethodById(id: string): Promise<boolean> {
    const uuid = new grpcTypes.UUID();
    uuid.setValue(id);
    const result = await this.call<grpcTypes.UUID, BoolValue>(
      this.client.testApiAccessMethodById,
      uuid,
    );
    return result.getValue();
  }

  public async testCustomApiAccessMethod(method: CustomProxy): Promise<boolean> {
    const result = await this.call<grpcTypes.CustomProxy, BoolValue>(
      this.client.testCustomApiAccessMethod,
      convertToCustomProxy(method),
    );
    return result.getValue();
  }

  public async applyJsonSettings(settings: string): Promise<void> {
    await this.callString(this.client.applyJsonSettings, settings);
  }

  public async clearAllRelayOverrides(): Promise<void> {
    await this.callEmpty(this.client.clearAllRelayOverrides);
  }

  public async setEnableRecents(enabled: boolean): Promise<void> {
    const boolValue = new BoolValue();
    boolValue.setValue(enabled);
    await this.call(this.client.setEnableRecents, boolValue);
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
      subscription.cancel();
    }
  }
}
