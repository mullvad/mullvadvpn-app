import jsonrpc from 'jsonrpc-lite';
import uuid from 'uuid';
import log from 'electron-log';

export default class Ipc {

  constructor(connectionString) {
    this.connectionString = connectionString;
  }

  on(event/*, listener*/) {
    log.info('Adding a listener to', event);
  }

  send(action, data) {
    const id = uuid.v4();
    const jsonrpcMessage = jsonrpc.request(id, action, data);

    return fetch(this.connectionString, {
      method: 'POST',
      headers: new Headers({
        'Content-Type': 'application/json',
      }),
      body: JSON.stringify(jsonrpcMessage),
    }).then((res) => {
      if (!res.ok) {
        throw {msg: 'HTTP request failed', data: res};
      }

      return res.json();
    }).then(json => {

      const c = jsonrpc.parseObject(json);
      if (c.type === 'error') {
        throw c.payload.error.message;
      }
      return c.payload.result;

    }).catch(e => {
      console.error('IPC call failed', action, data, e);
      throw e;
    });
  }
}
