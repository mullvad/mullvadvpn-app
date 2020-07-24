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
  BridgeState,
  ILocation,
  IAppVersionInfo,
  IAccountData,
} from '../shared/daemon-rpc-types';
import * as managementInterface from './management_interface/management_interface_grpc_pb';
import {
  AccountData,
  BridgeState as GrpcBridgeState,
  VoucherSubmission,
  GeoIpLocation,
  AccountHistory,
  AppVersionInfo,
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

  public async verifyWireguardKey(): Promise<boolean> {
    const response = await this.callEmpty<BoolValue>(this.client?.verifyWireguardKey);
    return response.getValue();
  }

  public async getVersionInfo(): Promise<IAppVersionInfo> {
    const response = await this.callEmpty<AppVersionInfo>(this.client?.getVersionInfo);
    return response.toObject();
  }
}
