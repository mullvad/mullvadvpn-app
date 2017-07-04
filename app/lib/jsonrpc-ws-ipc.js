// @flow

import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';
import log from 'electron-log';

export type UnansweredRequest = {
  resolve: (mixed) => void,
  reject: (mixed) => void,
  timerId: number,
  message: Object,
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
      result: mixed,
    }
  }
}
export type JsonRpcSuccess = {
  type: 'success',
  payload: {
    id: string,
    result: mixed,
  }
}
export type JsonRpcMessage = JsonRpcError | JsonRpcNotification | JsonRpcSuccess;

export class TimeOutError extends Error {
  jsonRpcMessage: Object;

  constructor(jsonRpcMessage: Object) {
    super('Request timed out');
    this.name = 'TimeOutError';
    this.jsonRpcMessage = jsonRpcMessage;
  }
}

export class InvalidReply extends Error {
  reply: mixed;

  constructor(reply: mixed, msg: ?string) {
    super(msg);
    this.name = 'InvalidReply';
    this.reply = reply;

    if(msg) {
      this.message = msg + ' - ';
    }
    this.message += JSON.stringify(reply);
  }
}

const DEFAULT_TIMEOUT_MILLIS = 750;

export default class Ipc {

  _connectionString: ?string;
  _onConnect: Array<{resolve: ()=>void}>;
  _unansweredRequests: {[string]: UnansweredRequest};
  _subscriptions: {[string|number]: (mixed) => void};
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

  on(event: string, listener: (mixed) => void): Promise<*> {

    log.info('Adding a listener to', event);
    return this.send(event + '_subscribe')
      .then(subscriptionId => {
        if (typeof subscriptionId === 'string' || typeof subscriptionId === 'number') {
          this._subscriptions[subscriptionId] = listener;
        } else {
          throw new InvalidReply(subscriptionId, 'The subscription id was not a string or a number');
        }
      })
      .catch(e => {
        log.error('Failed adding listener to', event, ':', e);
      });
  }

  send(action: string, ...data: Array<mixed>): Promise<mixed> {
    return new Promise((resolve, reject) => {
      const id = uuid.v4();

      const timerId = setTimeout(() => this._onTimeout(id), this._sendTimeoutMillis);
      const jsonrpcMessage = jsonrpc.request(id, action, data);
      this._unansweredRequests[id] = {
        resolve: resolve,
        reject: reject,
        timerId: timerId,
        message: jsonrpcMessage,
      };

      this._getWebSocket()
        .then(ws => {
          log.debug('Sending message', id, action);
          ws.send(jsonrpcMessage);
        })
        .catch(e => {
          log.error('Failed sending RPC message "' + action + '":', e);
          reject(e);
        });
    });
  }

  _getWebSocket() {
    return new Promise(resolve => {
      if (this._websocket && this._websocket.readyState === 1) { // Connected
        resolve(this._websocket);
      } else {
        log.debug('Waiting for websocket to connect');
        this._onConnect.push({
          resolve: () => resolve(this._websocket),
        });
      }
    });
  }

  _onTimeout(requestId) {
    const request = this._unansweredRequests[requestId];
    delete this._unansweredRequests[requestId];

    if (!request) {
      log.debug(requestId, 'timed out but it seems to already have been answered');
      return;
    }

    log.debug(request.message, 'timed out');
    request.reject(new TimeOutError(request.message));
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

    clearTimeout(request.timerId);

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
      const data = evt.data;
      if (typeof data === 'string') {
        this._onMessage(data);
      } else {
        log.error('Got invalid reply from the server', evt);
      }
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
