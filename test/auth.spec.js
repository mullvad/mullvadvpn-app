// @flow

import { expect } from 'chai';
import { setupIpcAndStore, setupBackendAndStore, failFast, checkNextTick } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import { Backend } from '../app/lib/backend';

describe('authentication', () => {

  it('authenticates before ipc call if unauthenticated', (done) => {
    const { store, mockIpc } = setupIpcAndStore();
    const credentials = {
      sharedSecret: 'foo',
      connectionString: '',
    };


    const chain = new IpcChain(mockIpc);
    chain.require('authenticate')
      .withInputValidation( secret => {
        expect(secret).to.equal(credentials.sharedSecret);
      })
      .done();

    chain.require('connect')
      .done();

    chain.onSuccessOrFailure(done);


    const backend = new Backend(store, credentials, mockIpc);
    backend.connect('example.com', 'udp', 1301);
  });

  it('reauthenticates on reconnect', (done) => {
    const { mockIpc, backend } = setupBackendAndStore();

    let authCount = 0;
    mockIpc.authenticate = () => {
      authCount++;
      return Promise.resolve();
    };


    mockIpc.killWebSocket();
    failFast(() => {
      expect(authCount).to.equal(0);
    }, done);


    backend.connect('example.com', 'udp', 1301);
    checkNextTick(() => {
      expect(authCount).to.equal(1);
    }, done);
  });
});
