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

  it('should send as soon as the websocket connects', (done) => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'hello') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 'ok')));
      }
    });

    transport
      .send('hello')
      .then(() => {
        done();
      })
      .catch((error) => {
        done(error);
      });

    transport.connect(WEBSOCKET_URL);
  });

  it('should reject failed jsonrpc requests', (done) => {
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

    transport.send('invalid-method').catch((error) => {
      try {
        expect(error.code).to.equal(-32601);
        expect(error.message).to.contain('Method not found');
        done();
      } catch (error) {
        done(error);
      }
    });

    transport.connect(WEBSOCKET_URL);
  });

  it('should route reply to correct promise', () => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'a message') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 'a reply')));
      }
    });

    const decoy = transport
      .send('a decoy', [], 100)
      .then(() => {
        throw new Error('Should not be called');
      })
      .catch((error) => {
        expect(error).to.be.an.instanceof(JsonRpcTransportTimeOutError);
      });

    const message = transport.send('a message', [], 100).then((reply) => {
      expect(reply).to.equal('a reply');
    });

    transport.connect(WEBSOCKET_URL);

    return Promise.all([message, decoy]);
  });

  it('should timeout if no response is returned', (done) => {
    transport
      .send('timeout-message', {}, 1)
      .then(() => {
        done(new Error('Should not be called'));
      })
      .catch((error) => {
        try {
          expect(error).to.be.an.instanceof(JsonRpcTransportTimeOutError);
          expect(error.message).to.contain('Request timed out');
          done();
        } catch (error) {
          done(error);
        }
      });

    transport.connect(WEBSOCKET_URL);
  });

  it('should route notifications', (done) => {
    server.on('message', (msg) => {
      const { payload } = jsonrpc.parse(msg);

      if (payload.method === 'event_subscribe') {
        server.send(JSON.stringify(jsonrpc.success(payload.id, 1)));
      }
    });

    transport
      .subscribe('event', (event) => {
        try {
          expect(event).to.equal('an event!');
          done();
        } catch (error) {
          done(error);
        }
      })
      .then(() => {
        server.send(
          JSON.stringify(jsonrpc.notification('event', { subscription: 1, result: 'an event!' })),
        );
      })
      .catch((error) => {
        done(error);
      });

    transport.connect(WEBSOCKET_URL);
  });
});
