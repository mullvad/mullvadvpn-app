// @flow

import Ipc from '../app/lib/jsonrpc-ws-ipc.js';
import jsonrpc from 'jsonrpc-lite';
import { expect } from 'chai';
import type { JsonRpcMessage } from '../app/lib/jsonrpc-ws-ipc.js';

describe('The IPC server', () => {

  it('should send as soon as the websocket connects', () => {
    const { ws, ipc } = setupIpc();
    ws.close();

    let sent = false;
    const p = ipc.send('hello')
      .then(() => {
        expect(sent).to.be.true;
      });

    ws.on('hello', (msg) => {
      sent = true;

      ws.replyOk(msg.id);
    });
    ws.acceptConnection();

    return p;
  });
});

function mockWebsocket() {
  const ws : any = {
    listeners: {},
    readyState: 1,
  };

  ws.on = (event, listener) => ws.listeners[event] = listener;
  ws.send = (data) => {
    const listener = ws.listeners[data.method];
    if (listener) {
      listener(data);
    }
  };

  ws.factory = () => ws;

  ws.acceptConnection = () => {
    ws.readyState = 1;
    ws.onopen();
  };
  ws.close = () => {
    ws.readyState = 3;
    ws.onclose();
  };

  ws.reply = (msg: JsonRpcMessage) => {
    ws.onmessage({data: JSON.stringify(msg)});
  };
  ws.replyOk = (id: string, msg) => {
    ws.reply(jsonrpc.success(id, msg || ''));
  };
  ws.replyFail = (id: string, msg: string, code: number) => {
    ws.reply(jsonrpc.error(id, new jsonrpc.JsonRpcError(msg, code)));
  };

  return ws;
}

function setupIpc() {
  const ws = mockWebsocket();
  return {
    ws: ws,
    ipc: new Ipc('1.2.3.4', ws.factory),
  };
}

