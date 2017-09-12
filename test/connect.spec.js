// @flow

import { expect } from 'chai';
import connectionActions from '../app/redux/connection/actions';
import { setupBackendAndStore, checkNextTick } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';

describe('connect', () => {

  it('should invoke set_custom_relay and then connect in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc);
    chain.require('setCustomRelay')
      .withInputValidation(
        (relayEndpoint) => {
          expect(relayEndpoint).to.equal(arbitraryRelay);
        },
      )
      .done();

    chain.require('connect')
      .done();

    chain.onSuccessOrFailure(done);

    store.dispatch(connectionActions.connect(backend, arbitraryRelay));
  });

  it('should set the connection state to \'disconnected\' on failed attempts', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    mockIpc.connect = () => new Promise((_, reject) => reject('Some error'));

    store.dispatch(connectionActions.connected());


    expect(store.getState().connection.status).not.to.equal('disconnected');

    store.dispatch(connectionActions.connect(backend, arbitraryRelay));


    checkNextTick(() => {
      expect(store.getState().connection.status).to.equal('disconnected');
    }, done);
  });

  it('should update the state with the server address', () => {
    const { store, backend } = setupBackendAndStore();
    const relay = {
      host: 'www.example.com',
      port: 1,
      protocol: 'udp',
    };

    return backend.connect(relay)
      .then( () => {
        const state = store.getState().connection;
        expect(state.status).to.equal('connecting');
        expect(state.serverAddress).to.equal('www.example.com');
      });
  });

  it('should correctly deduce \'connected\' from backend states', () => {
    const { store, mockIpc } = setupBackendAndStore();

    expect(store.getState().connection.status).not.to.equal('connected');
    mockIpc.sendNewState({ state: 'secured', target_state: 'secured' });
    expect(store.getState().connection.status).to.equal('connected');
  });

  it('should correctly deduce \'connecting\' from backend states', () => {
    const { store, mockIpc } = setupBackendAndStore();

    expect(store.getState().connection.status).not.to.equal('connecting');
    mockIpc.sendNewState({ state: 'unsecured', target_state: 'secured' });
    expect(store.getState().connection.status).to.equal('connecting');
  });

  it('should correctly deduce \'disconnected\' from backend states', () => {
    const { store, mockIpc } = setupBackendAndStore();
    store.dispatch(connectionActions.connected());

    expect(store.getState().connection.status).not.to.equal('disconnected');
    mockIpc.sendNewState({ state: 'unsecured', target_state: 'unsecured' });
    expect(store.getState().connection.status).to.equal('disconnected');
  });
});

const arbitraryRelay = {
  host: 'www.example.com',
  port: 1,
  protocol: 'udp',
};
