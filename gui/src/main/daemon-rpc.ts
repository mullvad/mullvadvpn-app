import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  DaemonEvent,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  ISettings,
  IWireguardPublicKey,
  KeygenEvent,
  RelaySettingsUpdate,
  TunnelState,
  VoucherErrorCode,
  VoucherResponse,
} from '../shared/daemon-rpc-types';

import { CommunicationError, InvalidAccountError } from './errors';
import { GrpcClient, ConnectionObserver, SubscriptionListener } from './grpc-client';

export { ConnectionObserver, SubscriptionListener } from './grpc-client';

export class ResponseParseError extends Error {
  constructor(message: string, private validationErrorValue?: Error) {
    super(message);
  }

  get validationError(): Error | undefined {
    return this.validationErrorValue;
  }
}

// Timeout used for RPC calls that do networking
// const NETWORK_CALL_TIMEOUT = 10000;

export class DaemonRpc {
  private transport = new GrpcClient();

  public connect(connectionParams: { path: string }): Promise<void> {
    return this.transport.connect(connectionParams);
  }

  public disconnect() {
    this.transport.disconnect();
  }

  public addConnectionObserver(observer: ConnectionObserver) {
    this.transport.addConnectionObserver(observer);
  }

  public removeConnectionObserver(observer: ConnectionObserver) {
    this.transport.removeConnectionObserver(observer);
  }

  public async getAccountData(accountToken: AccountToken): Promise<IAccountData> {
    try {
      return await this.transport.getAccountData(accountToken);
    } catch (error) {
      if (error.code) {
        switch (error.code) {
          case -200: // Account doesn't exist
            throw new InvalidAccountError();
          default:
          case -32603: // Internal error
            throw new CommunicationError();
        }
      } else {
        throw error;
      }
    }
  }

  public async getWwwAuthToken(): Promise<string> {
    return this.transport.getWwwAuthToken();
  }

  public async submitVoucher(voucherCode: string): Promise<VoucherResponse> {
    try {
      const response = await this.transport.submitVoucher(voucherCode);

      if (response.newExpiry) {
        return { type: 'success', new_expiry: response.newExpiry };
      }
    } catch (error) {
      if (error.code) {
        switch (error.code) {
          case VoucherErrorCode.Invalid:
            return { type: 'invalid' };
          case VoucherErrorCode.AlreadyUsed:
            return { type: 'already_used' };
        }
      }
    }

    return { type: 'error' };
  }

  public async getRelayLocations(): Promise<IRelayList> {
    const response = await this.transport.getRelayLocations();
    return { countries: response };
  }

  public async createNewAccount(): Promise<string> {
    return this.transport.createNewAccount();
  }

  public async setAccount(accountToken?: AccountToken): Promise<void> {
    await this.transport.setAccount(accountToken);
  }

  public async updateRelaySettings(_relaySettings: RelaySettingsUpdate): Promise<void> {
    // await this.transport.updateRelaySettings(relaySettings);
  }

  public async setAllowLan(allowLan: boolean): Promise<void> {
    await this.transport.setAllowLan(allowLan);
  }

  public async setShowBetaReleases(showBetaReleases: boolean): Promise<void> {
    await this.transport.setShowBetaReleases(showBetaReleases);
  }

  public async setEnableIpv6(enableIpv6: boolean): Promise<void> {
    await this.transport.setEnableIpv6(enableIpv6);
  }

  public async setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    await this.transport.setBlockWhenDisconnected(blockWhenDisconnected);
  }

  public async setBridgeState(bridgeState: BridgeState): Promise<void> {
    await this.transport.setBridgeState(bridgeState);
  }

  public async setBridgeSettings(_bridgeSettings: BridgeSettings): Promise<void> {
    // await this.transport.setBridgeSettings(bridgeSettings);
  }

  public async setOpenVpnMssfix(mssfix?: number): Promise<void> {
    await this.transport.setOpenVpnMssfix(mssfix);
  }

  public async setWireguardMtu(mtu?: number): Promise<void> {
    await this.transport.setWireguardMtu(mtu);
  }

  public async setAutoConnect(autoConnect: boolean): Promise<void> {
    await this.transport.setAutoConnect(autoConnect);
  }

  public async connectTunnel(): Promise<void> {
    await this.transport.connectTunnel();
  }

  public async disconnectTunnel(): Promise<void> {
    await this.transport.disconnectTunnel();
  }

  public async reconnectTunnel(): Promise<void> {
    await this.transport.reconnectTunnel();
  }

  public getLocation(): Promise<ILocation> {
    return this.transport.getLocation();
  }

  public getState(): Promise<TunnelState> {
    return this.transport.getState();
  }

  public getSettings(): Promise<ISettings> {
    return this.transport.getSettings();
  }

  public async subscribeDaemonEventListener(
    _listener: SubscriptionListener<DaemonEvent>,
  ): Promise<void> {
    // const subscriptionId = await this.transport.subscribe('daemon_event', (payload) => {
    //   let daemonEvent: DaemonEvent;
    //   try {
    //     daemonEvent = camelCaseObjectKeys(validate(daemonEventSchema, payload));
    //   } catch (error) {
    //     listener.onError(new ResponseParseError('Invalid payload from daemon_event', error));
    //     return;
    //   }
    //   listener.onEvent(daemonEvent);
    // });
    // listener.subscriptionId = subscriptionId;
  }

  public async unsubscribeDaemonEventListener(
    _listener: SubscriptionListener<DaemonEvent>,
  ): Promise<void> {
    // if (listener.subscriptionId) {
    //   return this.transport.unsubscribe('daemon_event', listener.subscriptionId);
    // }
  }

  public getAccountHistory(): Promise<AccountToken[]> {
    return this.transport.getAccountHistory();
  }

  public async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this.transport.removeAccountFromHistory(accountToken);
  }

  public getCurrentVersion(): Promise<string> {
    return this.transport.getCurrentVersion();
  }

  public generateWireguardKey(): Promise<KeygenEvent> {
    return this.transport.generateWireguardKey();
  }

  public getWireguardKey(): Promise<IWireguardPublicKey> {
    return this.transport.getWireguardKey();
  }

  public verifyWireguardKey(): Promise<boolean> {
    return this.transport.verifyWireguardKey();
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    return this.transport.getVersionInfo();
  }
}
