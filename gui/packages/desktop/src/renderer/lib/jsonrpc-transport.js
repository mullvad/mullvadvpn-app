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

const DEFAULT_TIMEOUT_MILLIS = 5000;

export default class JsonRpcTransport extends EventEmitter {
  _unansweredRequests: Map<string, UnansweredRequest> = new Map();
  _subscriptions: Map<string | number, (mixed) => void> = new Map();
  _transport: Transport;

  constructor(transport: Transport) {
    super();

    this._transport = transport;
  }

  /// Connect websocket
  connect(connectionParams: Object): Promise<void> {
    return new Promise((resolve, reject) => {
      this.disconnect();

      log.info('Connecting to transport with params', connectionParams);

      // A flag used to determine if Promise was resolved.
      let isPromiseResolved = false;

      const transport = this._transport;

      transport.set_on_open(() => {
        log.info('Transport is connected');
        this.emit('open');

        // Resolve the Promise
        resolve();
        isPromiseResolved = true;
      });

      transport.set_on_message((obj) => {
        this._onMessage(obj);
      });

      transport.set_on_close((error: ?Error) => {
        // Remove all subscriptions since they are connection based
        this._subscriptions.clear();

        this.emit('close', error);

        // Prevent rejecting a previously resolved Promise.
        if (!isPromiseResolved) {
          reject(error);
        }
      });
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
    log.silly(`Adding a listener to ${event}`);

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
        reject(new Error('Websocket is not connected.'));
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

interface Transport {
  close(): void;
  set_on_open(callback: (event: Event) => any): void;
  set_on_message(callback: (Object) => void): void;
  set_on_close(callback: (error: ?Error) => void): void;
  send(message: string): void;
  connect(params: Object): void;
}

export class WebsocketTransport implements Transport {
  ws: ?WebSocket;
  open_cb: any;
  message_cb: (Object) => void;
  close_cb: (event: ?Error) => void;

  constructor(ws: ?WebSocket) {
    this.ws = ws;
  }

  close() {
    if (this.ws) this.ws.close();
  }

  set_on_open(cb: (ev: Event) => void): void {
    this.open_cb = cb;
    if (this.ws) this.ws.onopen = cb;
  }

  set_on_message(cb: (Object) => void): void {
    this.message_cb = cb;
  }

  set_on_close(cb: (event: ?Error) => void): void {
    this.close_cb = cb;
  }

  send(msg: string) {
    this.ws && this.ws.send(msg);
  }

  connect(params: Object): void {
    if (!this.ws) {
      this.ws = new WebSocket(params.address);
    }
    this.ws.close();
    this.ws = new WebSocket(params.address);
    this.ws.onopen = (ev) => {
      this.open_cb && this.open_cb(ev);
    };
    this.ws.onmessage = (event) => {
      if (!this.message_cb) {
        return;
      }

      try {
        const data = event.data;
        if (typeof data === 'string') {
          const msg = JSON.parse(data);
          this.message_cb(msg);
        } else {
          throw event;
        }
      } catch (error) {
        log.error('Got invalid reply from server: ', error);
      }
    };
    this.ws.onclose = (event) => {
      if (!this.close_cb) {
        log.warn('No callback for capturing websocket closure set');
        return;
      }
      log.info(`The websocket connection closed with code: ${event.code}`);
      const error = event.code === 1000 ? null : new WebSocketError(event.code);
      this.close_cb(error);
    };
  }
}

// Given the correct parameters, this transport supports named pipes/unix
// domain sockets, and also TCP/UDP sockets
export class SocketTransport implements Transport {
  connection: net.Socket;
  message_cb: (message: Object) => void;
  close_cb: (error: ?Error) => void;
  open_cb: (event: Event) => void;
  // I guess I don't have type annotations for this
  json_stream: Object;
  error: ?Error;

  // Address can either be port number or
  constructor() {
    this.json_stream = JSONStream.parse();
  }

  async _connect(options: Object): Promise<void> {
    const connection = new net.Socket();

    connection.on('error', (err) => {
      if (this.close_cb !== null) {
        this.close_cb(err);
      }
      this.close();
    });

    connection.on('connect', (event) => {
      if (this.open_cb !== null) {
        this.open_cb(event);
      }
    });

    connection.connect(options);
    connection.pipe(this.json_stream);

    this.json_stream.on('data', (msg) => this._on_message(msg));

    this.json_stream.on('error', (err) => {
      if (this.close_cb !== null) {
        this.close_cb(err);
      }
      this.close();
    });

    this.connection = connection;
  }

  _on_error(error: ?Error) {
    this.close_cb(error);
    this.close();
  }

  _on_message(data: Object) {
    if (this.message_cb !== null) {
      this.message_cb(data);
    }
    try {
    } catch (error) {
      log.error('failed to parse JSON-RPC object: ', error);
    }
  }
  close() {
    if (this.connection) {
      this.connection.end();
      // resetting the parser
      this.json_stream = JSONStream.parse();
    }
  }

  set_on_open(cb: (event: Event) => void): void {
    this.open_cb = cb;
  }

  set_on_message(cb: (Object) => void): void {
    this.message_cb = cb;
  }

  set_on_close(cb: (event: ?Error) => void): void {
    this.close_cb = cb;
  }

  send(msg: string) {
    if (this.connection) {
      this.connection.write(msg);
    }
  }

  connect(options: Object): void {
    try {
      if (this.connection) {
        this.connection.end();
      }
    } catch (error) {
      log.error('failed to close the connection: ', error);
    }
    this._connect(options);
  }
}
