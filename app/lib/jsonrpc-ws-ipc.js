// @flow

import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';
import log from 'electron-log';

export type UnansweredRequest<T, E> = {
  resolve: (T) => void,
  reject: (E) => void,
  timeout: number,
}

export type JsonRpcError = {
  type: 'error',
  payload: {
    id: string,
    error: {
      message: string,
    }
  }
}
export type JsonRpcNotification = {
  type: 'notification',
  payload: {
    method: string,
    params: {
      subscription: string,
      result: any,
    }
  }
}
export type JsonRpcSuccess = {
  type: 'success',
  payload: {
    id: string,
    result: any,
  }
}
export type JsonRpcMessage = JsonRpcError | JsonRpcNotification | JsonRpcSuccess;

export class TimeOutError extends Error {
  constructor() {
    super('Request timed out');
    this.name = 'TimeOutError';
  }
}

const DEFAULT_TIMEOUT_MILLIS = 750;

export default class Ipc {

  _connectionString: ?string;
  _onConnect: Array<{resolve: ()=>void}>;
  _unansweredRequests: {[string]: UnansweredRequest<any, any>};
  _subscriptions: {[string]: (any) => void};
  _websocket: WebSocket;
  _backoff: ReconnectionBackoff;
  _websocketFactory: (string) => WebSocket;
  _sendTimeoutMillis: number;

  constructor(connectionString: string, websocketFactory: ?(string)=>WebSocket) {
    this._connectionString = connectionString;
    this._onConnect = [];
    this._unansweredRequests = {};
    this._subscriptions = {};
    this._websocketFactory = websocketFactory || (connectionString => new WebSocket(connectionString));
    this._sendTimeoutMillis = DEFAULT_TIMEOUT_MILLIS;

    this._backoff = new ReconnectionBackoff();
    this._reconnect();
  }

  setSendTimeout(millis: number) {
    this._sendTimeoutMillis = millis;
  }

  on(event: string, listener: (any) => void): Promise<*> {
    // We're currently not actually using the event parameter.
    // This is because we aren't sure if the backend will use
    // one subscription per event or one subscription per
    // event source.

    log.info('Adding a listener to', event);
    return this.send('event_subscribe')
      .then(subscriptionId => this._subscriptions[subscriptionId] = listener);
  }

  send(action: string, ...data: Array<any>): Promise<any> {
    return this._getWebSocket()
      .then(ws => this._send(ws, action, data))
      .catch(e => {
        log.error('Failed sending RPC message "' + action + '":', e);
        throw e;
      });
  }

  _getWebSocket() {
    return new Promise(resolve => {
      if (this._websocket.readyState === 1) { // Connected
        resolve(this._websocket);
      } else {
        log.debug('Waiting for websocket to connect');
        this._onConnect.push({
          resolve: () => resolve(this._websocket),
        });
      }
    });
  }

  _send(websocket, action, data) {
    return new Promise((resolve, reject) => {
      const id = uuid.v4();
      const jsonrpcMessage = jsonrpc.request(id, action, data);

      const timeout = setTimeout(() => this._onTimeout(id), this._sendTimeoutMillis);
      this._unansweredRequests[id] = {
        resolve: resolve,
        reject: reject,
        timeout: timeout,
      };
      log.debug('Sending message', id, action);
      websocket.send(jsonrpcMessage);
    });
  }

  _onTimeout(requestId) {
    const request = this._unansweredRequests[requestId];
    delete this._unansweredRequests[requestId];

    if (!request) {
      log.debug(requestId, 'timed out but it seems to already have been answered');
      return;
    }

    request.reject(new TimeOutError());
  }

  _onMessage(message: string) {
    const json = JSON.parse(message);
    const c = jsonrpc.parseObject(json);

    if (c.type === 'notification') {
      this._onNotification(c);
    } else {
      this._onReply(c);
    }
  }

  _onNotification(message: JsonRpcNotification) {
    const subscriptionId = message.payload.params.subscription;
    const listener = this._subscriptions[subscriptionId];

    if (listener) {
      log.debug('Got notification', message.payload.method, message.payload.params.result);
      listener(message.payload.params.result);
    } else {
      log.warn('Got notification for', message.payload.method, 'but no one is listening for it');
    }
  }

  _onReply(message: JsonRpcError | JsonRpcSuccess) {
    const id = message.payload.id;
    const request = this._unansweredRequests[id];
    delete this._unansweredRequests[id];

    if (!request) {
      log.warn('Got reply to', id, 'but no one was waiting for it');
      return;
    }

    log.debug('Got answer to', id, message.type);

    clearTimeout(request.timeout);

    if (message.type === 'error') {
      request.reject(message.payload.error);
    } else {
      const reply = message.payload.result;
      request.resolve(reply);
    }
  }

  _reconnect() {
    const connectionString = this._connectionString;
    if (!connectionString) return;

    log.info('Connecting to websocket', connectionString);
    this._websocket = this._websocketFactory(connectionString);

    this._websocket.onopen = () => {
      log.debug('Websocket is connected');
      this._backoff.successfullyConnected();

      while(this._onConnect.length > 0) {
        this._onConnect.pop().resolve();
      }
    };

    this._websocket.onmessage = (evt) => {
      const data: string = (evt.data: any);
      this._onMessage(data);
    };

    this._websocket.onclose = () => {
      const delay = this._backoff.getIncreasedBackoff();
      log.warn('The websocket connetion closed, attempting to reconnect it in', delay, 'milliseconds');
      setTimeout(() => this._reconnect(), delay);
    };
  }
}

/*
 * Used to calculate the time to wait before reconnecting
 * the websocket.
 *
 * It uses a linear backoff function that goes from 500ms
 * to 3000ms
 */
class ReconnectionBackoff {
  _attempt: number;

  constructor() {
    this._attempt = 0;
  }

  successfullyConnected() {
    this._attempt = 0;
  }

  getIncreasedBackoff() {
    if (this._attempt < 6) {
      this._attempt++;
    }

    return this._attempt * 500;
  }
}
