// @flow

import log from 'electron-log';
import { ConnectionObserver, SubscriptionListener, ResponseParseError } from '../daemon-rpc';
import type { DaemonRpcProtocol } from '../daemon-rpc';

export default class BaseSubscriptionProxy<T> {
  _rpc: DaemonRpcProtocol;
  _connectionObserver = new ConnectionObserver(
    () => {},
    () => {
      this._didDisconnectFromDaemon();
    },
  );

  _isSubscribed = false;
  _executingPromise: ?Promise<T>;

  _value: ?T;
  _onUpdate: (T) => void;

  constructor(rpc: DaemonRpcProtocol, onUpdate: (T) => void) {
    this._rpc = rpc;
    this._onUpdate = onUpdate;

    rpc.addConnectionObserver(this._connectionObserver);
  }

  async fetch(): Promise<T> {
    // return the cached promise if there is an ongoing fetch
    if (this._executingPromise) {
      return this._executingPromise;
    }

    // return the value if it's available
    if (this._value) {
      return this._value;
    }

    // subscribe if needed and fetch the initial state.
    const fetchPromise = this._subscribeAndFetchValue();

    // cache the fetch promise
    this._executingPromise = fetchPromise;

    try {
      const value = await fetchPromise;

      // cache the initial value
      this._value = value;

      // notify the delegate upon initial fetch
      this._onUpdate(value);

      return value;
    } catch (error) {
      throw error;
    } finally {
      // unset the cached fetch promise
      this._executingPromise = null;
    }
  }

  async _subscribeAndFetchValue(): Promise<T> {
    if (!this._isSubscribed) {
      await this._subscribeValueListener();
      this._isSubscribed = true;
    }

    // request the initial value
    return await this.constructor.requestValue(this._rpc);
  }

  static subscribeValueListener(
    _rpc: DaemonRpcProtocol,
    _listener: SubscriptionListener<T>,
  ): Promise<void> {
    throw new Error(
      `Override static ${this.constructor.name}.subscribeValueListener() in subclasses`,
    );
  }

  static requestValue(_rpc: DaemonRpcProtocol): Promise<T> {
    throw new Error(`Override static ${this.constructor.name}.requestValue() in subclasses`);
  }

  _subscribeValueListener(): Promise<void> {
    const listener = new SubscriptionListener(
      (value: T) => {
        this._didReceiveUpdate(value);
      },
      (error: Error) => {
        let reason = '';

        if (error instanceof ResponseParseError) {
          const validationError = error.validationError;
          if (validationError) {
            reason = ` Reason: ${validationError.message}`;
          }
        }

        log.error(`Failed to deserialize the payload: ${error.message}.${reason}`);
      },
    );
    return this.constructor.subscribeValueListener(this._rpc, listener);
  }

  _didReceiveUpdate(updatedValue: T) {
    this._value = updatedValue;
    this._onUpdate(updatedValue);
  }

  _didDisconnectFromDaemon() {
    this._isSubscribed = false;
    this._executingPromise = null;
    this._value = null;
  }
}
