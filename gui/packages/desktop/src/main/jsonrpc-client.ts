import assert from 'assert';
import { EventEmitter } from 'events';
import log from 'electron-log';
import jsonrpc from 'jsonrpc-lite';
import * as uuid from 'uuid';
import * as net from 'net';
import JSONStream from 'JSONStream';

export type UnansweredRequest = {
  resolve: (value: any) => void;
  reject: (value: any) => void;
  timerId: NodeJS.Timeout;
  message: Object;
};

export type JsonRpcErrorResponse = {
  type: 'error';
  payload: {
    id: string;
    error: {
      code: number;
      message: string;
    };
  };
};
export type JsonRpcNotification = {
  type: 'notification';
  payload: {
    method: string;
    params: {
      subscription: string;
      result: any;
    };
  };
};
export type JsonRpcSuccess = {
  type: 'success';
  payload: {
    id: string;
    result: any;
  };
};
export type JsonRpcMessage = JsonRpcErrorResponse | JsonRpcNotification | JsonRpcSuccess;

export class RemoteError extends Error {
  _code: number;
  _details: string;

  constructor(code: number, details: string) {
    super(`Remote JSON-RPC error ${code}: ${details}`);
    this._code = code;
    this._details = details;
  }

  get code(): number {
    return this._code;
  }

  get details(): string {
    return this._details;
  }
}

export class TimeOutError extends Error {
  _jsonRpcMessage: Object;

  constructor(jsonRpcMessage: Object) {
    super('Request timed out');

    this._jsonRpcMessage = jsonRpcMessage;
  }

  get jsonRpcMessage(): Object {
    return this._jsonRpcMessage;
  }
}

export class SubscriptionError extends Error {
  _reply: any;

  constructor(message: string, reply: any) {
    const replyString = JSON.stringify(reply);

    super(`${message}: ${replyString}`);

    this._reply = reply;
  }

  get reply(): any {
    return this._reply;
  }
}

export class WebSocketError extends Error {
  _code: number;

  constructor(code: number) {
    super(WebSocketError.reason(code));
    this._code = code;
  }

  get code(): number {
    return this._code;
  }

  static reason(code: number): string {
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
}

export class TransportError extends Error {}

const DEFAULT_TIMEOUT_MILLIS = 5000;

export default class JsonRpcClient<T> extends EventEmitter {
  _unansweredRequests: Map<string, UnansweredRequest> = new Map();
  _subscriptions: Map<string | number, (value: any) => void> = new Map();
  _transport: Transport<T>;

  constructor(transport: Transport<T>) {
    super();

    this._transport = transport;
  }

  /// Connect websocket
  connect(connectionParams: T): Promise<void> {
    return new Promise((resolve, reject) => {
      this.disconnect();

      log.info('Connecting to transport with params', connectionParams);

      // A flag used to determine if Promise was resolved.
      let isPromiseResolved = false;

      const transport = this._transport;

      transport.onOpen = () => {
        log.info('Transport is connected');
        this.emit('open');

        // Resolve the Promise
        resolve();
        isPromiseResolved = true;
      };

      transport.onMessage = (obj) => {
        this._onMessage(obj);
      };

      transport.onClose = (error?: Error) => {
        // Remove all subscriptions since they are connection based
        this._subscriptions.clear();

        this.emit('close', error);

        // Prevent rejecting a previously resolved Promise.
        if (!isPromiseResolved) {
          reject(error);
        }
      };
      transport.connect(connectionParams);

      this._transport = transport;
    });
  }

  disconnect() {
    if (this._transport) {
      this._transport.close();
    }
  }

  async subscribe(event: string, listener: (value: any) => void): Promise<void> {
    log.silly(`Adding a listener for ${event}`);

    try {
      const subscriptionId = await this.send(`${event}_subscribe`);
      if (typeof subscriptionId === 'string' || typeof subscriptionId === 'number') {
        this._subscriptions.set(subscriptionId, listener);
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

  send(action: string, data?: any, timeout: number = DEFAULT_TIMEOUT_MILLIS): Promise<any> {
    return new Promise((resolve, reject) => {
      const transport = this._transport;
      if (!transport) {
        reject(new Error('RPC client transport is not connected.'));
        return;
      }

      const id = uuid.v4();
      const payload = this._prepareParams(data);
      const timerId = setTimeout(() => this._onTimeout(id), timeout);
      const message = jsonrpc.request(id, action, payload);
      this._unansweredRequests.set(id, {
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
        this._unansweredRequests.delete(id);
        clearTimeout(timerId);

        throw error;
      }
    });
  }

  _prepareParams(data?: any): Array<any> | Object {
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

  _onTimeout(requestId: string) {
    const request = this._unansweredRequests.get(requestId);

    this._unansweredRequests.delete(requestId);

    if (request) {
      log.warn(`Request ${requestId} timed out: `, request.message);
      request.reject(new TimeOutError(request.message));
    } else {
      log.warn(`Request ${requestId} timed out but it seems to already have been answered`);
    }
  }

  _onMessage(obj: Object) {
    let messages: Array<any> = [];
    try {
      // TODO: Fix the type weirdness.
      // @ts-ignore
      const message = jsonrpc.parseObject(obj);
      messages = Array.isArray(message) ? message : [message];
    } catch (error) {
      log.error(`Failed to parse JSON-RPC message: ${error} for object`);
    }

    for (const message of messages) {
      if (message.type === 'notification') {
        this._onNotification(message);
      } else {
        this._onReply(message);
      }
    }
  }

  _onNotification(message: JsonRpcNotification) {
    const subscriptionId = message.payload.params.subscription;
    const listener = this._subscriptions.get(subscriptionId);

    if (listener) {
      log.silly(`Got notification for ${message.payload.method}`);
      listener(message.payload.params.result);
    } else {
      log.warn(`Got notification for ${message.payload.method} but no one is listening for it`);
    }
  }

  _onReply(message: JsonRpcErrorResponse | JsonRpcSuccess) {
    const id = message.payload.id;
    const request = this._unansweredRequests.get(id);
    this._unansweredRequests.delete(id);

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

interface Transport<T> {
  close(): void;
  onOpen: () => void;
  onMessage: (data: Object) => void;
  onClose: (error?: Error) => void;
  send(message: string): void;
  connect(params: T): void;
}

export class WebsocketTransport implements Transport<string> {
  ws?: WebSocket;
  onOpen = () => {};
  onMessage = (_message: Object) => {};
  onClose = (_error?: Error) => {};

  constructor(ws?: WebSocket) {
    this.ws = ws;
  }

  close() {
    if (this.ws) this.ws.close();
  }

  send(msg: string) {
    if (this.ws) {
      this.ws.send(msg);
    }
  }

  connect(params: string): void {
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
export class SocketTransport implements Transport<{ path: string }> {
  onMessage = (_message: Object) => {};
  onClose = (_error?: Error) => {};
  onOpen = () => {};

  _connection?: net.Socket;
  _socketReady = false;
  _shouldClose = false;
  _lastError?: Error;

  connect(options: { path: string }) {
    assert(!this._connection, 'Make sure to close the existing socket');

    const jsonStream = JSONStream.parse(null)
      .on('data', this._onJsonStreamData)
      .on('error', this._onJsonStreamError);

    const connection = new net.Socket()
      .on('ready', this._onSocketReady)
      .on('error', this._onSocketError)
      .on('close', this._onSocketClose);

    this._connection = connection;
    this._socketReady = false;
    this._shouldClose = false;
    this._lastError = undefined;

    log.debug('Connect socket');

    connection.pipe(jsonStream);
    connection.connect(options);
  }

  close() {
    this._shouldClose = true;

    try {
      if (this._connection) {
        this._connection.end();
      }
    } catch (error) {
      log.error('Failed to close the socket: ', error);
    }

    this._connection = undefined;
  }

  send(msg: string) {
    if (this._socketReady && this._connection) {
      this._connection.write(msg);
    } else {
      throw new TransportError('Socket not connected');
    }
  }

  _onSocketReady = () => {
    this._socketReady = true;

    log.debug('Socket is ready');

    this.onOpen();
  };

  _onSocketError = (error: Error) => {
    this._lastError = error;

    log.error('Socket error: ', error);
  };

  _onSocketClose = (hadError: boolean) => {
    if (this._shouldClose) {
      log.debug(`Socket was closed deliberately`);

      this.onClose();
    } else if (hadError) {
      log.debug(`Socket was closed due to an error`);

      this.onClose(this._lastError);
    } else {
      log.debug(`Socket was closed by peer`);

      this.onClose(new TransportError('Socket was closed by peer'));
    }
  };

  _onJsonStreamData = (data: Object) => {
    this.onMessage(data);
  };

  _onJsonStreamError = (error: Error) => {
    log.error('Socket JSON stream error: ', error);

    if (this._connection) {
      // This will destroy the socket and emit "error" and "close" events
      this._connection.destroy(error);
    }
  };
}
