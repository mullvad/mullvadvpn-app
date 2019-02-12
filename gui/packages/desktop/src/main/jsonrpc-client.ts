import assert from 'assert';
import log from 'electron-log';
import { EventEmitter } from 'events';
import jsonrpc from 'jsonrpc-lite';
import JSONStream from 'JSONStream';
import * as net from 'net';
import * as uuid from 'uuid';

export interface IUnansweredRequest {
  resolve: (value: any) => void;
  reject: (value: any) => void;
  timerId: NodeJS.Timeout;
  message: object;
}

export interface IJsonRpcErrorResponse {
  type: 'error';
  payload: {
    id: string;
    error: {
      code: number;
      message: string;
    };
  };
}
export interface IJsonRpcNotification {
  type: 'notification';
  payload: {
    method: string;
    params: {
      subscription: string;
      result: any;
    };
  };
}
export interface IJsonRpcSuccess {
  type: 'success';
  payload: {
    id: string;
    result: any;
  };
}
export type JsonRpcMessage = IJsonRpcErrorResponse | IJsonRpcNotification | IJsonRpcSuccess;

export class RemoteError extends Error {
  constructor(private codeValue: number, private detailsValue: string) {
    super(`Remote JSON-RPC error ${codeValue}: ${detailsValue}`);
  }

  get code(): number {
    return this.codeValue;
  }

  get details(): string {
    return this.detailsValue;
  }
}

export class TimeOutError extends Error {
  constructor(private jsonRpcMessageValue: object) {
    super('Request timed out');
  }

  get jsonRpcMessage(): object {
    return this.jsonRpcMessageValue;
  }
}

export class SubscriptionError extends Error {
  constructor(message: string, private replyValue: any) {
    super(`${message}: ${JSON.stringify(replyValue)}`);
  }

  get reply(): any {
    return this.replyValue;
  }
}

export class WebSocketError extends Error {
  get code(): number {
    return this.codeValue;
  }

  private static reason(code: number): string {
    switch (code) {
      case 1006:
        return 'Abnormal closure';
      case 1011:
        return 'Internal error';
      case 1012:
        return 'Service restart';
      case 1014:
        return 'Bad gateway';
      default:
        return `Unknown (${code})`;
    }
  }
  constructor(private codeValue: number) {
    super(WebSocketError.reason(codeValue));
  }
}

export class TransportError extends Error {}

const DEFAULT_TIMEOUT_MILLIS = 5000;

export default class JsonRpcClient<T> extends EventEmitter {
  private unansweredRequests: Map<string, IUnansweredRequest> = new Map();
  private subscriptions: Map<string | number, (value: any) => void> = new Map();
  private transport: ITransport<T>;

  constructor(transport: ITransport<T>) {
    super();

    this.transport = transport;
  }

  /// Connect websocket
  public connect(connectionParams: T): Promise<void> {
    return new Promise((resolve, reject) => {
      this.disconnect();

      log.info('Connecting to transport with params', connectionParams);

      // A flag used to determine if Promise was resolved.
      let isPromiseResolved = false;

      const transport = this.transport;

      transport.onOpen = () => {
        log.info('Transport is connected');
        this.emit('open');

        // Resolve the Promise
        resolve();
        isPromiseResolved = true;
      };

      transport.onMessage = (obj) => {
        this.onMessage(obj);
      };

      transport.onClose = (error?: Error) => {
        // Remove all subscriptions since they are connection based
        this.subscriptions.clear();

        this.emit('close', error);

        // Prevent rejecting a previously resolved Promise.
        if (!isPromiseResolved) {
          reject(error);
        }
      };
      transport.connect(connectionParams);

      this.transport = transport;
    });
  }

  public disconnect() {
    if (this.transport) {
      this.transport.close();
    }
  }

  public async subscribe(event: string, listener: (value: any) => void): Promise<void> {
    log.silly(`Adding a listener for ${event}`);

    try {
      const subscriptionId = await this.send(`${event}_subscribe`);
      if (typeof subscriptionId === 'string' || typeof subscriptionId === 'number') {
        this.subscriptions.set(subscriptionId, listener);
      } else {
        throw new SubscriptionError(
          'The subscription id was not a string or a number',
          subscriptionId,
        );
      }
    } catch (e) {
      log.error(`Failed adding listener to ${event}: ${e.message}`);
      throw e;
    }
  }

  public send(action: string, data?: any, timeout: number = DEFAULT_TIMEOUT_MILLIS): Promise<any> {
    return new Promise((resolve, reject) => {
      const transport = this.transport;
      if (!transport) {
        reject(new Error('RPC client transport is not connected.'));
        return;
      }

      const id = uuid.v4();
      const payload = this.prepareParams(data);
      const timerId = setTimeout(() => this.onTimeout(id), timeout);
      const message = jsonrpc.request(id, action, payload);
      this.unansweredRequests.set(id, {
        resolve,
        reject,
        timerId,
        message,
      });

      try {
        log.silly('Sending message', id, action);
        transport.send(JSON.stringify(message));
      } catch (error) {
        log.error(`Failed sending RPC message "${action}": ${error.message}`);

        // clean up on error
        this.unansweredRequests.delete(id);
        clearTimeout(timerId);

        throw error;
      }
    });
  }

  private prepareParams(data?: any): any[] | object {
    // JSONRPC only accepts arrays and objects as params, but
    // this isn't very nice to use, so this method wraps other
    // types in an array. The choice of array is based on try-and-error

    if (data === undefined) {
      return [];
    } else if (data === null) {
      return [null];
    } else if (Array.isArray(data) || typeof data === 'object') {
      return data;
    } else {
      return [data];
    }
  }

  private onTimeout(requestId: string) {
    const request = this.unansweredRequests.get(requestId);

    this.unansweredRequests.delete(requestId);

    if (request) {
      log.warn(`Request ${requestId} timed out: `, request.message);
      request.reject(new TimeOutError(request.message));
    } else {
      log.warn(`Request ${requestId} timed out but it seems to already have been answered`);
    }
  }

  private onMessage(obj: object) {
    let message: any;
    try {
      // @ts-ignore
      message = jsonrpc.parseObject(obj);
    } catch (error) {
      log.error(`Failed to parse JSON-RPC message: ${error} for object`);
    }

    if (message.type === 'notification') {
      this.onNotification(message);
    } else {
      this.onReply(message);
    }
  }

  private onNotification(message: IJsonRpcNotification) {
    const subscriptionId = message.payload.params.subscription;
    const listener = this.subscriptions.get(subscriptionId);

    if (listener) {
      log.silly(`Got notification for ${message.payload.method}`);
      listener(message.payload.params.result);
    } else {
      log.warn(`Got notification for ${message.payload.method} but no one is listening for it`);
    }
  }

  private onReply(message: IJsonRpcErrorResponse | IJsonRpcSuccess) {
    const id = message.payload.id;
    const request = this.unansweredRequests.get(id);
    this.unansweredRequests.delete(id);

    if (request) {
      log.silly('Got answer to', id, message.type);

      clearTimeout(request.timerId);

      if (message.type === 'error') {
        const error = message.payload.error;
        request.reject(new RemoteError(error.code, error.message));
      } else {
        const reply = message.payload.result;
        request.resolve(reply);
      }
    } else {
      log.warn(`Got reply to ${id} but no one was waiting for it`);
    }
  }
}

interface ITransport<T> {
  onOpen: () => void;
  onMessage: (data: object) => void;
  onClose: (error?: Error) => void;
  close(): void;
  send(message: string): void;
  connect(params: T): void;
}

export class WebsocketTransport implements ITransport<string> {
  public ws?: WebSocket;

  constructor(ws?: WebSocket) {
    this.ws = ws;
  }
  public onOpen = () => {
    // no-op
  };
  public onMessage = (_message: object) => {
    // no-op
  };
  public onClose = (_error?: Error) => {
    // no-op
  };

  public close() {
    if (this.ws) {
      this.ws.close();
    }
  }

  public send(msg: string) {
    if (this.ws) {
      this.ws.send(msg);
    }
  }

  public connect(params: string): void {
    if (this.ws) {
      this.ws.close();
    }
    this.ws = new WebSocket(params);
    this.ws.onopen = (_event) => {
      this.onOpen();
    };
    this.ws.onmessage = (event) => {
      try {
        const data = event.data;
        if (typeof data === 'string') {
          const msg = JSON.parse(data);
          this.onMessage(msg);
        } else {
          throw event;
        }
      } catch (error) {
        log.error('Got invalid reply from server: ', error);
      }
    };

    this.ws.onclose = (event) => {
      log.info(`The websocket connection closed with code: ${event.code}`);
      if (event.code === 1000) {
        this.onClose();
      } else {
        this.onClose(new WebSocketError(event.code));
      }
    };
  }
}

// Given the correct parameters, this transport supports named pipes/unix
// domain sockets, and also TCP/UDP sockets
export class SocketTransport implements ITransport<{ path: string }> {
  private connection?: net.Socket;
  private jsonStream?: NodeJS.ReadWriteStream;
  private socketReady = false;
  private lastError?: Error;
  public onMessage = (_message: object) => {
    // no-op
  };
  public onClose = (_error?: Error) => {
    // no-op
  };
  public onOpen = () => {
    // no-op
  };

  public connect(options: { path: string }) {
    assert(!this.connection, 'Make sure to close the existing socket');

    const jsonStream = JSONStream.parse(null)
      .on('data', this.onJsonStreamData)
      .once('error', this.onJsonStreamError);

    const connection = new net.Socket()
      .once('ready', this.onSocketReady)
      .once('error', this.onSocketError)
      .once('close', this.onSocketClose);

    this.connection = connection;
    this.jsonStream = jsonStream;
    this.socketReady = false;
    this.lastError = undefined;

    log.debug('Connect socket');

    connection.pipe(jsonStream);
    connection.connect(options);
  }

  public close() {
    if (this.connection) {
      log.debug('Close socket');

      // closing socket is not synchronous, so remove all of the event handlers first
      this.connection
        .removeListener('ready', this.onSocketReady)
        .removeListener('error', this.onSocketError)
        .removeListener('close', this.onSocketClose);

      this.jsonStream!.removeListener('data', this.onJsonStreamData).removeListener(
        'error',
        this.onJsonStreamError,
      );

      try {
        this.connection.end();
      } catch (error) {
        log.error('Failed to close the socket: ', error);
      }

      this.connection = undefined;
      this.jsonStream = undefined;
      this.onClose();
    }
  }

  public send(msg: string) {
    if (this.socketReady && this.connection) {
      this.connection.write(msg);
    } else {
      throw new TransportError('Socket not connected');
    }
  }

  private onSocketReady = () => {
    this.socketReady = true;

    log.debug('Socket is ready');

    this.onOpen();
  };

  private onSocketError = (error: Error) => {
    this.lastError = error;

    log.error('Socket error: ', error);
  };

  private onSocketClose = (hadError: boolean) => {
    if (hadError) {
      log.debug(`Socket was closed due to an error: `, this.lastError);

      this.onClose(this.lastError);
    } else {
      log.debug(`Socket was closed by peer`);

      this.onClose(new TransportError('Socket was closed by peer'));
    }
  };

  private onJsonStreamData = (data: object) => {
    this.onMessage(data);
  };

  private onJsonStreamError = (error: Error) => {
    log.error('Socket JSON stream error: ', error);

    if (this.connection) {
      // This will destroy the socket and emit "error" and "close" events
      this.connection.destroy(error);
    }
  };
}
