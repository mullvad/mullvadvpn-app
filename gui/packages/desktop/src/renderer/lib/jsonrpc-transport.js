// @flow

import { EventEmitter } from 'events';
import log from 'electron-log';
import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';

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

export class ConnectionError extends Error {
  _code: number;

  constructor(code: number) {
    super(ConnectionError.reason(code));
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
  _webSocket: ?WebSocket;
  _websocketFactory: (string) => WebSocket;

  constructor(websocketFactory: ?(string) => WebSocket) {
    super();
    this._websocketFactory =
      websocketFactory || ((connectionString) => new WebSocket(connectionString));
  }

  /// Connect websocket
  connect(connectionString: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.disconnect();

      log.info('Connecting to websocket', connectionString);

      const webSocket = this._websocketFactory(connectionString);

      // A flag used to determine if Promise was resolved.
      let isPromiseResolved = false;

      webSocket.onopen = () => {
        log.info('Websocket is connected');
        this.emit('open');

        // Resolve the Promise
        resolve();
        isPromiseResolved = true;
      };

      webSocket.onmessage = (event) => {
        const data = event.data;
        if (typeof data === 'string') {
          this._onMessage(data);
        } else {
          log.error('Got invalid reply from the server', event);
        }
      };

      webSocket.onclose = (event) => {
        log.info(`The websocket connection closed with code: ${event.code}`);

        // Remove all subscriptions since they are connection based
        this._subscriptions.clear();

        // 1000 is a code used for normal connection closure.
        const connectionError = event.code === 1000 ? null : new ConnectionError(event.code);

        this.emit('close', connectionError);

        // Prevent rejecting a previously resolved Promise.
        if (!isPromiseResolved) {
          reject(connectionError);
        }
      };

      this._webSocket = webSocket;
    });
  }

  disconnect() {
    if (this._webSocket) {
      this._webSocket.close();
      this._webSocket = null;
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
      const webSocket = this._webSocket;
      if (!webSocket) {
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
        webSocket.send(JSON.stringify(message));
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

  _onMessage(message: string) {
    const result = jsonrpc.parse(message);
    const messages = Array.isArray(result) ? result : [result];

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
