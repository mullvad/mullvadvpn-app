import * as grpc from '@grpc/grpc-js';
import fs from 'fs';
import { Empty } from 'google-protobuf/google/protobuf/empty_pb.js';
import {
  BoolValue,
  StringValue,
  UInt32Value,
} from 'google-protobuf/google/protobuf/wrappers_pb.js';
import { ManagementServiceClient } from 'management-interface';
import { promisify } from 'util';

import log from '../shared/logging';

const NETWORK_CALL_TIMEOUT = 10000;
const CHANNEL_STATE_TIMEOUT = 1000 * 60 * 60;

type CallFunctionArgument<T, R> =
  | ((arg: T, callback: (error: Error | null, result: R) => void) => void)
  | undefined;

export const noConnectionError = new Error('No connection established to daemon');

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

export class GrpcClient {
  protected client: ManagementServiceClient;
  private isConnectedValue = false;
  private isClosed = false;
  private reconnectionTimeout?: NodeJS.Timeout;
  private isUds: boolean;

  constructor(
    private rpcPath: string,
    private connectionObserver?: ConnectionObserver,
  ) {
    this.isUds = rpcPath.startsWith('unix://');
    this.client = new ManagementServiceClient(rpcPath, grpc.credentials.createInsecure());
  }

  public get isConnected() {
    return this.isConnectedValue;
  }

  public reopen(connectionObserver?: ConnectionObserver) {
    if (this.isClosed) {
      this.isClosed = false;
      this.client = new ManagementServiceClient(this.rpcPath, grpc.credentials.createInsecure());

      this.connectionObserver = connectionObserver;
    }
  }

  public connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const usedClient = this.client;
      this.client.waitForReady(this.deadlineFromNow(), (error) => {
        if (this.client !== usedClient) {
          reject(new Error('Stale connection attempt'));
          return;
        }

        if (error) {
          this.onClose(error);
          this.ensureConnectivity();
          reject(error);
        } else if (this.isUds) {
          this.verifyOwnership()
            .then(() => {
              this.onOpen();
              resolve();
            })
            .catch((error) => {
              this.onClose(error);
              this.ensureConnectivity();
              reject(error);
            });
        } else {
          this.onOpen();
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

  protected callEmpty<R = Empty>(fn: CallFunctionArgument<Empty, R>): Promise<R> {
    return this.call<Empty, R>(fn, new Empty());
  }

  protected callString<R = Empty>(
    fn: CallFunctionArgument<StringValue, R>,
    value?: string,
  ): Promise<R> {
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

  private onOpen() {
    this.reconnectionTimeout = undefined;
    this.isConnectedValue = true;
    this.connectionObserver?.onOpen();
    this.setChannelCallback();
  }

  private deadlineFromNow() {
    return Date.now() + NETWORK_CALL_TIMEOUT;
  }

  private channelStateTimeout(): number {
    return Date.now() + CHANNEL_STATE_TIMEOUT;
  }

  private onClose(error?: Error) {
    const wasConnected = this.isConnectedValue;
    this.isConnectedValue = false;

    this.connectionObserver?.onClose(wasConnected, error);
  }

  private channelOptions(): grpc.ClientOptions {
    return {
      'grpc.max_reconnect_backoff_ms': 3000,
      'grpc.initial_reconnect_backoff_ms': 3000,
      'grpc.keepalive_time_ms': Math.pow(2, 30),
      'grpc.keepalive_timeout_ms': Math.pow(2, 30),
      'grpc.client_idle_timeout_ms': Math.pow(2, 30),
    };
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
    if (this.reconnectionTimeout) {
      clearTimeout(this.reconnectionTimeout);
    }
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

  // Assert that the gRPC connection is owned by an administrator
  private async verifyOwnership() {
    const rpcPath = this.rpcPath.replace(/^unix:\/\//, '');

    if (process.platform === 'win32') {
      try {
        const { pipeIsAdminOwned } = await import('windows-utils');
        pipeIsAdminOwned(rpcPath);
      } catch (e) {
        if (e && typeof e === 'object' && 'message' in e) {
          throw new Error(`Failed to verify admin ownership of named pipe. ${e.message}`);
        } else {
          throw new Error('Failed to verify admin ownership of named pipe');
        }
      }
      log.info('Verified pipe ownership');
    } else {
      const stat = fs.statSync(rpcPath);
      if (stat.uid !== 0) {
        throw new Error('Failed to verify root ownership of socket');
      }
      log.info('Verified socket ownership');
    }
  }
}
