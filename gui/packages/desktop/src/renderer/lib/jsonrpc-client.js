// @flow

import { EventEmitter } from 'events';
import log from 'electron-log';
import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';
import net from 'net';
import JSONStream from 'JSONStream';

export type UnansweredRequest = {
  resolve: (mixed) => void,
  reject: (mixed) => void,
  timerId: TimeoutID,
  message: Object,
};

export type JsonRpcErrorResponse = {
  type: 'error',
  payload: {
    id: string,
    error: {
      code: number,
      message: string,
    },
  },
};
export type JsonRpcNotification = {
  type: 'notification',
  payload: {
    method: string,
    params: {
      subscription: string,
      result: mixed,
    },
  },
};
export type JsonRpcSuccess = {
  type: 'success',
  payload: {
    id: string,
    result: mixed,
  },
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
  _reply: mixed;

  constructor(message: string, reply: mixed) {
    const replyString = JSON.stringify(reply);

    super(`${message}: ${replyString}`);

    this._reply = reply;
  }

  get reply(): mixed {
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
  _subscriptions: Map<string | number, (mixed) => void> = new Map();
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

      transport.onClose = (error: ?Error) => {
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

  async subscribe(event: string, listener: (mixed) => void): Promise<*> {
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

  send(action: string, data: mixed, timeout: number = DEFAULT_TIMEOUT_MILLIS): Promise<mixed> {
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

  _prepareParams(data: mixed): Array<mixed> | Object {
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

  _onTimeout(requestId) {
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
    let messages = [];
    try {
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
      log.silly('Got notification', message.payload.method, message.payload.params.result);
      listener(message.payload.params.result);
    } else {
      log.warn('Got notification for', message.payload.method, 'but no one is listening for it');
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
  onOpen: (event: Event) => void;
  onMessage: (Object) => void;
  onClose: (error: ?Error) => void;
  send(message: string): void;
  connect(params: T): void;
}

export class WebsocketTransport implements Transport<string> {
  ws: ?WebSocket;
  onOpen: (event: Event) => void;
  onMessage: (Object) => void;
  onClose: (error: ?Error) => void;

  constructor(ws: ?WebSocket) {
    this.ws = ws;
    this.onOpen = () => {};
    this.onMessage = () => {};
    this.onClose = () => {};
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
    this.ws.onopen = this.onOpen;
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
      const error = event.code === 1000 ? null : new WebSocketError(event.code);
      this.onClose(error);
    };
  }
}

// Given the correct parameters, this transport supports named pipes/unix
// domain sockets, and also TCP/UDP sockets
export class SocketTransport implements Transport<{ path: string }> {
  connection: ?net.Socket;
  onMessage: (message: Object) => void;
  onClose: (error: ?Error) => void;
  onOpen: (event: Event) => void;
  socketClosed: boolean;
  shouldClose: boolean;

  constructor() {
    this.connection = null;
    this.onMessage = () => {};
    this.onClose = () => {};
    this.onOpen = () => {};
    this.socketClosed = false;
    this.shouldClose = false;
  }

  _connect(options: { path: string }) {
    const connection = new net.Socket();

    connection.on('error', (err) => {
      this._fail(err);
    });

    connection.on('close', (hadErr) => {
      // if there's no error but nobody expected the socket to be closed an
      // error should still be propagated
      let err = null;
      if (!this.shouldClose || hadErr) {
        err = new TransportError('socket closed unexpectedly');
      }
      this._fail(err);
    });

    connection.on('connect', (event) => {
      this.connection = connection;
      this.onOpen(event);
    });

    const jsonStream = JSONStream.parse();

    connection.pipe(jsonStream);

    jsonStream.on('data', this.onMessage);

    jsonStream.on('error', (err) => {
      this._fail(err);
    });

    this.socketClosed = false;
    this.shouldClose = false;
    connection.connect(options);
  }

  _fail(err: ?Error) {
    if (!this.socketClosed) {
      this.socketClosed = true;
      this.onClose(err);
      this.close();
    }
  }

  close() {
    this.shouldClose = true;
    try {
      if (this.connection) {
        this.connection.end();
      }
    } catch (error) {
      log.error('failed to close the connection: ', error);
    }
    this.connection = null;
  }

  send(msg: string) {
    if (this.connection) {
      this.connection.write(msg);
    } else {
      throw new TransportError('Socket not connected');
    }
  }

  connect(options: { path: string }): void {
    this._connect(options);
  }
}
