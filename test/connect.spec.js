// @flow

import { expect } from 'chai';
import connectionActions from '../app/redux/connection/actions';
import { setupBackendAndStore, checkNextTick } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';

describe('connect', () => {

  it('should invoke set_country and then connect in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc);
    chain.require('setCountry')
      .withInputValidation(
        (country) => expect(country).to.equal('example.com')
      )
      .done();

    chain.require('connect')
      .done();

    chain.onSuccessOrFailure(done);

    store.dispatch(connectionActions.connect(backend, 'example.com'));
  });

  it('should set the connection state to \'disconnected\' on failed attempts', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    mockIpc.connect = () => new Promise((_, reject) => reject('Some error'));

    store.dispatch(connectionActions.connected());


    expect(store.getState().connection.status).not.to.equal('disconnected');

    store.dispatch(connectionActions.connect(backend, 'example.com'));


    checkNextTick(() => {
      expect(store.getState().connection.status).to.equal('disconnected');
    }, done);
  });

  it('should update the state with the server address', () => {
    const { store, backend } = setupBackendAndStore();
    const arbitraryString = 'www.example.com';

    return backend.connect(arbitraryString)
      .then( () => {
        const state = store.getState().connection;
        expect(state.status).to.equal('connecting');
        expect(state.serverAddress).to.equal(arbitraryString);
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

