// @flow

import { expect } from 'chai';
import connectionActions from '../app/redux/connection/actions';
import { setupBackendAndStore, checkNextTick } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';

describe('connect', () => {

  it('should invoke set_country and then connect in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc, done);
    chain.addRequiredStep('setCountry')
      .withInputValidation(
        (country) => expect(country).to.equal('example.com')
      )
      .done();

    chain.addRequiredStep('connect')
      .done();

    store.dispatch(connectionActions.connect(backend, 'example.com'));
  });

  it('should set the connection state to \'disconnected\' on failed attempts', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    mockIpc.connect = () => new Promise((_, reject) => reject('Some error'));

    store.dispatch(connectionActions.connectionChange({
      status: 'connected',
    }));


    expect(store.getState().connection.status).not.to.equal('disconnected');

    store.dispatch(connectionActions.connect(backend, 'example.com'));


    checkNextTick(() => {
      expect(store.getState().connection.status).to.equal('disconnected');
    }, done);
  });

  it('should update the store on \'secured\' state from the backend', () => {
    const { store, mockIpc } = setupBackendAndStore();

    expect(store.getState().connection.status).not.to.equal('connected');
    mockIpc.sendNewState('secured');
    expect(store.getState().connection.status).to.equal('connected');

  });

  it('should update the store on \'unsecured\' state from the backend', () => {
    const { store, mockIpc } = setupBackendAndStore();
    store.dispatch(connectionActions.connectionChange({
      status: 'connected',
    }));

    expect(store.getState().connection.status).not.to.equal('disconnected');
    mockIpc.sendNewState('unsecured');
    expect(store.getState().connection.status).to.equal('disconnected');

  });
});

