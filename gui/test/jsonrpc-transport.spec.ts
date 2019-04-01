import { expect } from 'chai';
import { it, describe, beforeEach } from 'mocha';
import jsonrpc from 'jsonrpc-lite';
import JsonRpcClient, { ITransport, TimeOutError } from '../src/main/jsonrpc-client';

describe('JSON RPC transport', () => {
  let client: JsonRpcClient<string>, transport: MockTransport;

  beforeEach(() => {
    transport = new MockTransport();
    client = new JsonRpcClient(transport);
    return client.connect('');
  });

  it('should reject failed jsonrpc requests', async () => {
    transport.onServerMessage = (msg) => {
      const parsedMessage = jsonrpc.parseObject(msg);

      if (parsedMessage.type === 'request' && parsedMessage.payload.method === 'invalid-method') {
        transport.reply(
          JSON.stringify(
            jsonrpc.error(
              parsedMessage.payload.id,
              new jsonrpc.JsonRpcError('Method not found', -32601),
            ),
          ),
        );
      }
    };

    const sendPromise = client.send('invalid-method');

    return expect(sendPromise).to.eventually.be.rejectedWith('Method not found');
  });

  it('should route reply to correct promise', async () => {
    transport.onServerMessage = (msg) => {
      const parsedMessage = jsonrpc.parseObject(msg);

      if (parsedMessage.type === 'request' && parsedMessage.payload.method === 'a message') {
        transport.reply(JSON.stringify(jsonrpc.success(parsedMessage.payload.id, 'a reply')));
      }
    };

    const decoyPromise = client.send('a decoy', [], 100);
    const messagePromise = client.send('a message', [], 100);

    return Promise.all([
      expect(messagePromise).to.eventually.be.equal('a reply'),
      expect(decoyPromise).to.eventually.be.rejectedWith(TimeOutError),
    ]);
  });

  it('should timeout if no response is returned', async () => {
    const sendPromise = client.send('timeout-message', {}, 1);

    return expect(sendPromise).to.eventually.be.rejectedWith(TimeOutError, 'Request timed out');
  });

  it('should route notifications', async () => {
    transport.onServerMessage = (msg) => {
      const parsedMessage = jsonrpc.parseObject(msg);

      if (parsedMessage.type === 'request' && parsedMessage.payload.method === 'event_subscribe') {
        transport.reply(JSON.stringify(jsonrpc.success(parsedMessage.payload.id, 1)));
      }
    };

    const eventPromiseHelper = (() => {
      let borrowedResolve: ((param: any) => void) | undefined = undefined;
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

    await client.subscribe('event', eventPromiseHelper.resolve);

    transport.reply(
      JSON.stringify(jsonrpc.notification('event', { subscription: 1, result: 'beacon' })),
    );

    return expect(eventPromiseHelper.promise).to.eventually.be.equal('beacon');
  });
});

class MockTransport implements ITransport<string> {
  public onOpen = () => {
    // no-op
  };
  public onMessage = (_message: object) => {
    // no-op
  };
  public onServerMessage = (_message: object) => {
    // no-op
  };
  public onClose = (_error?: Error) => {
    // no-op
  };

  public close() {
    this.onClose();
  }

  public send(msg: string) {
    this.onServerMessage(JSON.parse(msg));
  }

  public reply(msg: string) {
    this.onMessage(JSON.parse(msg));
  }

  public connect(_params: string) {
    this.onOpen();
  }
}
