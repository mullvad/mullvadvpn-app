// @flow

import Ipc from '../app/lib/jsonrpc-ws-ipc.js';
import jsonrpc from 'jsonrpc-lite';
import { expect } from 'chai';
import assert from 'assert';
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

  it('should reject failed jsonrpc requests', () => {
    const { ws, ipc } = setupIpc();
    ws.on('WHAT_IS_THIS', (msg) => {
      ws.replyFail(msg.id, 'Method not found', -32601);
    });

    return ipc.send('WHAT_IS_THIS')
      .catch((e) => {
        expect(e.code).to.equal(-32601);
        expect(e.message).to.contain('Method not found');
      });
  });

  it('should route reply to correct promise', () => {
    const { ws, ipc } = setupIpc();

    ws.on('a message', (msg) => ws.replyOk(msg.id, 'a reply'));

    const decoy = ipc.send('a decoy')
      .then(() => assert(false, 'Should not be called'))
      .catch(e => {
        if (e.name !== 'TimeOutError') {
          throw e;
        }
      });
    const message = ipc.send('a message')
      .then((reply) => expect(reply).to.equal('a reply'));

    return Promise.all([message, decoy]);
  });

  it('should timeout if no response is returned', () => {
    const { ipc } = setupIpc();

    return ipc.send('a message', [], 1)
      .catch((e) => {
        expect(e.name).to.equal('TimeOutError');
        expect(e.message).to.contain('timed out');
      });
  });

  it('should route notifications', (done) => {
    const { ws, ipc } = setupIpc();

    const eventListener = (event) => {
      try {
        expect(event).to.equal('an event!');
        done();
      } catch (ex) {
        done(ex);
      }
    };

    ws.on('event_subscribe', (msg) => ws.replyOk(msg.id, 1));
    ipc.on('event', eventListener)
      .then(() => {
        ws.reply(jsonrpc.notification('event', {subscription:1, result: 'an event!'}));
      })
      .catch((e) => done(e));
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

