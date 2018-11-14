// @flow

import { ipcRenderer } from 'electron';
import log from 'electron-log';
import uuid from 'uuid';

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

import {
  NoCreditError,
  NoInternetError,
  NoDaemonError,
  InvalidAccountError,
  CommunicationError,
} from '../../main/errors';

import { TimeOutError, RemoteError } from '../../main/jsonrpc-client';

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

type ErrorInfo = {
  className: string,
  data: Object,
};

export default class DaemonRpcProxy implements DaemonRpcProtocol {
  _connectionObservers: Array<ConnectionObserver> = [];
  _stateListeners: Array<SubscriptionListener<TunnelStateTransition>> = [];
  _settingsListeners: Array<SubscriptionListener<Settings>> = [];

  constructor() {
    ipcRenderer.on('connected-daemon', () => {
      for (const observer of this._connectionObservers) {
        observer._onOpen();
      }
    });

    ipcRenderer.on('disconnected-daemon', (_event: Event, error: ?Error) => {
      for (const observer of this._connectionObservers) {
        observer._onClose(error);
      }
    });

    ipcRenderer.on('state-changed', (_event: Event, newState: TunnelStateTransition) => {
      for (const listener of this._stateListeners) {
        listener._onEvent(newState);
      }
    });

    ipcRenderer.on('settings-changed', (_event: Event, newSettings: Settings) => {
      for (const listener of this._settingsListeners) {
        listener._onEvent(newSettings);
      }
    });
  }

  connect(_connectionInfo: { path: string }): void {
    throw new Error('Do not call this method.');
  }

  disconnect(): void {
    throw new Error('Do not call this method.');
  }

  getAccountData(accountToken: AccountToken): Promise<AccountData> {
    return this._sendMessage('getAccountData', accountToken);
  }

  getRelayLocations(): Promise<RelayList> {
    return this._sendMessage('getRelayLocations');
  }

  setAccount(accountToken: ?AccountToken): Promise<void> {
    return this._sendMessage('setAccount', accountToken);
  }

  updateRelaySettings(update: RelaySettingsUpdate): Promise<void> {
    return this._sendMessage('updateRelaySettings', update);
  }

  setAllowLan(allowLan: boolean): Promise<void> {
    return this._sendMessage('setAllowLan', allowLan);
  }

  setEnableIpv6(enableIpv6: boolean): Promise<void> {
    return this._sendMessage('setEnableIpv6', enableIpv6);
  }

  setOpenVpnMssfix(mssfix: ?number): Promise<void> {
    return this._sendMessage('setOpenVpnMssfix', mssfix);
  }

  setAutoConnect(autoConnect: boolean): Promise<void> {
    return this._sendMessage('setAutoConnect', autoConnect);
  }

  connectTunnel(): Promise<void> {
    return this._sendMessage('connectTunnel');
  }

  disconnectTunnel(): Promise<void> {
    return this._sendMessage('disconnectTunnel');
  }

  getLocation(): Promise<Location> {
    return this._sendMessage('getLocation');
  }

  getState(): Promise<TunnelStateTransition> {
    return this._sendMessage('getState');
  }

  getSettings(): Promise<Settings> {
    return this._sendMessage('getSettings');
  }

  subscribeStateListener(listener: SubscriptionListener<TunnelStateTransition>): Promise<void> {
    this._stateListeners.push(listener);
    return Promise.resolve();
  }

  subscribeSettingsListener(listener: SubscriptionListener<Settings>): Promise<void> {
    this._settingsListeners.push(listener);
    return Promise.resolve();
  }

  addConnectionObserver(observer: ConnectionObserver): void {
    this._connectionObservers.push(observer);
  }

  removeConnectionObserver(observer: ConnectionObserver): void {
    const index = this._connectionObservers.indexOf(observer);
    if (index !== -1) {
      this._connectionObservers.splice(index, 1);
    }
  }

  getAccountHistory(): Promise<Array<AccountToken>> {
    return this._sendMessage('getAccountHistory');
  }

  removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    return this._sendMessage('removeAccountFromHistory', accountToken);
  }

  getCurrentVersion(): Promise<string> {
    return this._sendMessage('getCurrentVersion');
  }

  getVersionInfo(): Promise<AppVersionInfo> {
    return this._sendMessage('getVersionInfo');
  }

  _sendMessage<T, R>(method: string, payload: ?T): Promise<R> {
    const promise: Promise<R> = new Promise((resolve, reject) => {
      const id = uuid.v4();

      ipcRenderer.once(
        `daemon-rpc-reply-${id}`,
        (_event: Event, result: R, errorInfo: ?ErrorInfo) => {
          log.debug(
            `Got daemon-rpc-reply: ${id} ${method} ${JSON.stringify(result)} ${JSON.stringify(
              errorInfo,
            )}`,
          );

          if (errorInfo) {
            const error = this._deserializeError(errorInfo.className, errorInfo.data);
            log.debug(`Deserialized an error to instance of ${error.constructor.name}`);
            reject(error);
          } else {
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
