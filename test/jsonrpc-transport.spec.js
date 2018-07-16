// @flow

import JsonRpcTransport, {
  TimeOutError as JsonRpcTransportTimeOutError,
} from '../app/lib/jsonrpc-transport';
import jsonrpc from 'jsonrpc-lite';
import { Server, WebSocket as MockWebSocket } from 'mock-socket';

describe('JSON RPC transport', () => {
  const WEBSOCKET_URL = 'ws://localhost:8080';
  let server: Server, transport: JsonRpcTransport;

  beforeEach(() => {
    server = new Server(WEBSOCKET_URL);
    transport = new JsonRpcTransport((s) => new MockWebSocket(s));
  });

  afterEach(() => {
    server.close();
  });

  it('should send as soon as the websocket connects', () => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'hello') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 'ok')));
      }
    });

    const sendPromise = transport.send('hello');

    transport.connect(WEBSOCKET_URL);

    return expect(sendPromise).to.eventually.be.fulfilled;
  });

  it('should reject failed jsonrpc requests', () => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'invalid-method') {
        server.send(
          JSON.stringify(
            jsonrpc.error(payload.id, new jsonrpc.JsonRpcError('Method not found', -32601)),
          ),
        );
      }
    });

    const sendPromise = transport.send('invalid-method');

    transport.connect(WEBSOCKET_URL);

    return expect(sendPromise).to.eventually.be.rejectedWith('Method not found');
  });

  it('should route reply to correct promise', () => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'a message') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 'a reply')));
      }
    });

    const decoyPromise = transport.send('a decoy', [], 100);
    const messagePromise = transport.send('a message', [], 100);

    transport.connect(WEBSOCKET_URL);

    return Promise.all([
      expect(messagePromise).to.eventually.be.equal('a reply'),
      expect(decoyPromise).to.eventually.be.rejectedWith(JsonRpcTransportTimeOutError),
    ]);
  });

  it('should timeout if no response is returned', () => {
    const sendPromise = transport.send('timeout-message', {}, 1);

    transport.connect(WEBSOCKET_URL);

    return expect(sendPromise).to.eventually.be.rejectedWith(
      JsonRpcTransportTimeOutError,
      'Request timed out',
    );
  });

  it('should route notifications', () => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'event_subscribe') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 1)));
      }
    });

    transport.connect(WEBSOCKET_URL);

    let subscribePromise;
    const eventPromise = new Promise((resolve) => {
      subscribePromise = transport.subscribe('event', resolve).then((value) => {
        server.send(
          JSON.stringify(jsonrpc.notification('event', { subscription: 1, result: 'beacon' })),
        );
        return value;
      });
    });

    return Promise.all([
      expect(subscribePromise).to.eventually.be.fulfilled,
      expect(eventPromise).to.eventually.be.equal('beacon'),
    ]);
  });
});
