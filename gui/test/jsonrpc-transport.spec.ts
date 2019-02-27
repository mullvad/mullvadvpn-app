import { expect } from 'chai';
import { it, describe, beforeEach, afterEach } from 'mocha';
import jsonrpc from 'jsonrpc-lite';
import { Server } from 'mock-socket';
import JsonRpcClient, { WebsocketTransport, TimeOutError } from '../src/main/jsonrpc-client';

describe('JSON RPC transport', () => {
  const WEBSOCKET_URL = 'ws://localhost:8080';
  let server: Server, transport: JsonRpcClient<string>;

  beforeEach(() => {
    server = new Server(WEBSOCKET_URL);
    transport = new JsonRpcClient(new WebsocketTransport());
  });

  afterEach(() => {
    server.close();
  });

  it('should reject failed jsonrpc requests', async () => {
    server.on('connection', (socket) => {
      socket.on('message', (msg) => {
        const { payload } = jsonrpc.parse(msg);
        if (payload.method === 'invalid-method') {
          socket.send(
            JSON.stringify(
              jsonrpc.error(payload.id, new jsonrpc.JsonRpcError('Method not found', -32601)),
            ),
          );
        }
      });
    });

    await transport.connect(WEBSOCKET_URL);
    const sendPromise = transport.send('invalid-method');

    return expect(sendPromise).to.eventually.be.rejectedWith('Method not found');
  });

  it('should route reply to correct promise', async () => {
    server.on('connection', (socket) => {
      socket.on('message', (msg) => {
        const { payload } = jsonrpc.parse(msg);
        if (payload.method === 'a message') {
          socket.send(JSON.stringify(jsonrpc.success(payload.id, 'a reply')));
        }
      });
    });

    await transport.connect(WEBSOCKET_URL);

    const decoyPromise = transport.send('a decoy', [], 100);
    const messagePromise = transport.send('a message', [], 100);

    return Promise.all([
      expect(messagePromise).to.eventually.be.equal('a reply'),
      expect(decoyPromise).to.eventually.be.rejectedWith(TimeOutError),
    ]);
  });

  it('should timeout if no response is returned', async () => {
    await transport.connect(WEBSOCKET_URL);
    const sendPromise = transport.send('timeout-message', {}, 1);

    return expect(sendPromise).to.eventually.be.rejectedWith(TimeOutError, 'Request timed out');
  });

  it('should route notifications', async () => {
    server.on('connection', (socket) => {
      socket.on('message', (msg) => {
        const { payload } = jsonrpc.parse(msg);
        if (payload.method === 'event_subscribe') {
          socket.send(JSON.stringify(jsonrpc.success(payload.id, 1)));
        }
      });
    });

    await transport.connect(WEBSOCKET_URL);

    const eventPromiseHelper = (() => {
      let borrowedResolve: (param: any) => void | undefined = undefined;
      const promise = new Promise((resolve) => (borrowedResolve = resolve));
      /* Flow does not understand that the body of Promise runs immediately.
         see https://github.com/facebook/flow/issues/6711 */
      if (!borrowedResolve) {
        throw new Error();
      }
      return {
        resolve: borrowedResolve,
        promise,
      };
    })();

    await transport.subscribe('event', eventPromiseHelper.resolve);

    server.emit(
      'message',
      JSON.stringify(jsonrpc.notification('event', { subscription: 1, result: 'beacon' })),
    );

    return expect(eventPromiseHelper.promise).to.eventually.be.equal('beacon');
  });
});
