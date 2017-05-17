import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';
import log from 'electron-log';

export default class Ipc {

  constructor(connectionString) {
    this._connectionString = connectionString;
    this._onConnect = [];
    this._unansweredRequests = {};

    this._reconnect();
  }

  on(event/*, listener*/) {
    log.info('Adding a listener to', event);
  }

  send(action, ...data) {
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

      this._unansweredRequests[id] = {resolve: resolve, reject: reject};
      log.debug('Sending message', id, action);
      websocket.send(jsonrpcMessage);
    });
  }

  _onMessage(message) {
    const json = JSON.parse(message);
    const c = jsonrpc.parseObject(json);

    const id = c.payload.id;
    const request = this._unansweredRequests[id];
    delete this._unansweredRequests[id];

    log.debug('Got answer to', id, c.type);
    if (c.type === 'error') {
      request.reject(c.payload.error.message);
    } else {
      const reply = c.payload.result;
      request.resolve(reply);
    }
  }

  _reconnect() {
    if (!this._connectionString) return;

    log.info('Connecting to websocket', this._connectionString);
    this._websocket = new WebSocket(this._connectionString);

    this._websocket.onopen = () => {
      log.debug('Websocket is connected');
      while(this._onConnect.length > 0) {
        this._onConnect.pop().resolve();
      }
    };

    this._websocket.onmessage = (evt) => {
      this._onMessage(evt.data);
    };

    this._websocket.onclose = () => {
      log.warn('The websocket connetion closed, attempting to reconnect it');
      this._reconnect();
    };
  }
}
