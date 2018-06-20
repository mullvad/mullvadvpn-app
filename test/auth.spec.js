// @flow

import { setupIpcAndStore, setupBackendAndStore } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import { Backend } from '../app/lib/backend';

describe('authentication', () => {
  it('authenticates before ipc call if unauthenticated', (done) => {
    const { store, mockIpc } = setupIpcAndStore();

    const credentials = {
      connectionString: 'ws://localhost:1234/',
      sharedSecret: '1234',
    };

    const chain = new IpcChain(mockIpc);
    chain.expect('authenticate').withInputValidation((secret) => {
      expect(secret).to.equal(credentials.sharedSecret);
    });
    chain.expect('connect');
    chain.end(done);

    const backend = new Backend(store, mockIpc);
    backend.connect(credentials);

    backend.connectTunnel();
  });

  it('reauthenticates on reconnect', async () => {
    const { mockIpc, backend } = setupBackendAndStore();

    mockIpc.authenticate = spy(mockIpc.authenticate);
    await mockIpc.connectTunnel();
    mockIpc.killWebSocket();

    expect(mockIpc.authenticate).to.not.have.been.called();

    await backend.connectTunnel();
    expect(mockIpc.authenticate).to.have.been.called.once;
  });
});
