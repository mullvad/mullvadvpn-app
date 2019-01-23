// @flow

import { ipcRenderer } from 'electron';
import log from 'electron-log';
import uuid from 'uuid';

// Re-export types
export type {
  AccountToken,
  AccountData,
  AppVersionInfo,
  Settings,
  TunnelStateTransition,
  RelayList,
  RelaySettingsUpdate,
  RelaySettings,
  RelaySettingsCustom,
  RelaySettingsNormalUpdate,
  RelayLocation,
  RelayProtocol,
  Ip,
  Location,
  TunnelEndpoint,
  BlockReason,
  AfterDisconnect,
  ResponseParseError,
  TunnelState,
  DaemonRpcProtocol,
} from '../../main/daemon-rpc';

export { ConnectionObserver, SubscriptionListener } from '../../main/daemon-rpc';

import type {
  AccountToken,
  AccountData,
  AppVersionInfo,
  DaemonRpcProtocol,
  RelaySettingsUpdate,
  RelayList,
  TunnelStateTransition,
  Settings,
  Location,
} from '../../main/daemon-rpc';

import { ConnectionObserver, SubscriptionListener } from '../../main/daemon-rpc';

import {
  NoCreditError,
  NoInternetError,
  NoDaemonError,
  InvalidAccountError,
  CommunicationError,
} from '../../main/errors';

import { TimeOutError, RemoteError } from '../../main/jsonrpc-client';

type ErrorInfo = {
  className: string,
  data: Object,
};

export default class DaemonRpcProxy implements DaemonRpcProtocol {
  connect(_connectionInfo: { path: string }): void {
    throw new Error('Do not call this method.');
  }

  disconnect(): void {
    throw new Error('Do not call this method.');
  }

  getAccountData(_accountToken: AccountToken): Promise<AccountData> {
    throw new Error('Do not call this method.');
  }

  getRelayLocations(): Promise<RelayList> {
    throw new Error('Do not call this method.');
  }

  setAccount(_accountToken: ?AccountToken): Promise<void> {
    throw new Error('Do not use this method');
  }

  updateRelaySettings(_update: RelaySettingsUpdate): Promise<void> {
    throw new Error('Do not use this method');
  }

  setAllowLan(allowLan: boolean): Promise<void> {
    throw new Error('Do not call this method.');
  }

  setEnableIpv6(enableIpv6: boolean): Promise<void> {
    throw new Error('Do not call this method.');
  }

  setBlockWhenDisconnected(blockWhenDisconnected: boolean): Promise<void> {
    throw new Error('Do not call this method.');
  }

  setOpenVpnMssfix(mssfix: ?number): Promise<void> {
    throw new Error('Do not call this method.');
  }

  setAutoConnect(_autoConnect: boolean): Promise<void> {
    throw new Error('Do not call this method.');
  }

  connectTunnel(): Promise<void> {
    throw new Error('Do not call this method.');
  }

  disconnectTunnel(): Promise<void> {
    throw new Error('Do not call this method.');
  }

  getLocation(): Promise<Location> {
    throw new Error('Do not call this method.');
  }

  getState(): Promise<TunnelStateTransition> {
    throw new Error('Do not call this method.');
  }

  getSettings(): Promise<Settings> {
    throw new Error('Do not call this method.');
  }

  subscribeStateListener(_listener: SubscriptionListener<TunnelStateTransition>): Promise<void> {
    throw new Error('Do not call this method.');
  }

  subscribeSettingsListener(_listener: SubscriptionListener<Settings>): Promise<void> {
    throw new Error('Do not call this method.');
  }

  addConnectionObserver(_observer: ConnectionObserver): void {
    throw new Error('Do not call this method.');
  }

  removeConnectionObserver(_observer: ConnectionObserver): void {
    throw new Error('Do not call this method.');
  }

  getAccountHistory(): Promise<Array<AccountToken>> {
    throw new Error('Do not use this method');
  }

  removeAccountFromHistory(_accountToken: AccountToken): Promise<void> {
    throw new Error('Do not use this method');
  }

  getCurrentVersion(): Promise<string> {
    throw new Error('Do not use this method');
  }

  getVersionInfo(): Promise<AppVersionInfo> {
    throw new Error('Do not use this method');
  }

  _sendMessage<T, R>(method: string, payload: ?T): Promise<R> {
    const promise: Promise<R> = new Promise((resolve, reject) => {
      const id = uuid.v4();

      ipcRenderer.once(
        `daemon-rpc-reply-${id}`,
        (_event: Event, result: R, errorInfo: ?ErrorInfo) => {
          if (errorInfo) {
            const error = this._deserializeError(errorInfo.className, errorInfo.data);

            log.debug(
              `Got daemon-rpc-reply-${id} ${method} with error: ${JSON.stringify(errorInfo)}`,
            );

            reject(error);
          } else {
            log.debug(`Got daemon-rpc-reply-${id} ${method} with success`);
            resolve(result);
          }
        },
      );

      ipcRenderer.send(`daemon-rpc-call`, id, method, payload);
    });

    return promise;
  }

  _deserializeError(className: string, data: Object): Error {
    switch (className) {
      case 'RemoteError':
        return new RemoteError(data.code, data.details);
      case 'TimeOutError':
        return new TimeOutError(data._jsonRpcMessage);
      case 'NoCreditError':
        return new NoCreditError();
      case 'NoInternetError':
        return new NoInternetError();
      case 'NoDaemonError':
        return new NoDaemonError();
      case 'InvalidAccountError':
        return new InvalidAccountError();
      case 'CommunicationError':
        return new CommunicationError();
      default:
        return new Error(data.message || '');
    }
  }
}
