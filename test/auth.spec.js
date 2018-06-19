// @flow

import { setupIpcAndStore, setupBackendAndStore } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import { Backend } from '../app/lib/backend';

describe('authentication', () => {
  it('authenticates before ipc call if unauthenticated', (done) => {
    const { store, mockIpc } = setupIpcAndStore();

    const chain = new IpcChain(mockIpc);
    chain.onSuccessOrFailure(done);
    chain.expect('authenticate').withInputValidation((secret) => {
      expect(secret).to.equal(credentials.sharedSecret);
    });
    chain.expect('connect');

    const credentials = {
      sharedSecret: '',
      connectionString: '',
    };
    const backend = new Backend(store, credentials, mockIpc);
    backend.connect();
  });

  it('reauthenticates on reconnect', async () => {
    const { mockIpc, backend } = setupBackendAndStore();

    mockIpc.authenticate = spy(mockIpc.authenticate);
    mockIpc.killWebSocket();

    expect(mockIpc.authenticate).to.not.have.been.called();

    await backend.connect();
    expect(mockIpc.authenticate).to.have.been.called.once;
  });
});
