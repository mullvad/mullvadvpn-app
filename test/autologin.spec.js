// @flow

import { expect } from 'chai';
import { setupBackendAndStore, setupBackendAndMockStore, getLocation } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';

describe('autologin', () => {

  it('should send get_account then get_account_data if an account is set', (done) => {
    const { mockIpc, backend } = setupBackendAndStore();

    const randomAccountToken = '12345';

    const chain = new IpcChain(mockIpc);
    chain.require('getAccount')
      .withReturnValue(randomAccountToken)
      .done();

    chain.require('getAccountData')
      .withInputValidation((num) => {
        expect(num).to.equal(randomAccountToken);
      })
      .done();

    chain.onSuccessOrFailure(done);

    backend.autologin();
  });

  it('should redirect to the login page if no account is set', () => {
    const { store, backend, mockIpc } = setupBackendAndMockStore();

    mockIpc.getAccount = () => new Promise((_, reject) => reject('NO_ACCOUNT'));

    return backend.autologin()
      .then( () => {
        expect(getLocation(store)).to.equal('/');
      })
      .catch( (e) => {
        if (e !== 'NO_ACCOUNT') {
          throw e;
        }
      });
  });

  it('should redirect to the login page for non-existing accounts', () => {
    const { store, backend, mockIpc } = setupBackendAndMockStore();

    mockIpc.getAccount = () => new Promise(r => r('123'));
    mockIpc.getAccountData = () => new Promise((_, reject) => reject('NO_ACCOUNT'));

    return backend.autologin()
      .then( () => {
        expect(getLocation(store)).to.equal('/');
      })
      .catch( (e) => {
        if (e !== 'NO_ACCOUNT') {
          throw e;
        }
      });
  });

  it('should mark the state as not logged in if no account is set', () => {
    const { store, backend, mockIpc } = setupBackendAndStore();

    mockIpc.getAccount = () => Promise.resolve(null);

    return backend.autologin()
      .catch( () => {}) // ignore errors
      .then( () => {
        const state = store.getState().account;

        expect(state.status).to.equal('none');
        expect(state.accountToken).to.be.null;
        expect(state.error).to.be.null;
      });
  });

  it('should mark the state as not logged in for non-existing accounts', () => {
    const { store, backend, mockIpc } = setupBackendAndStore();

    mockIpc.getAccount = () => new Promise(r => r('123'));
    mockIpc.getAccountData = () => new Promise((_, reject) => reject('NO ACCOUNT'));

    return backend.autologin()
      .catch( () => {}) // ignore errors
      .then( () => {
        const state = store.getState().account;

        expect(state.status).to.equal('none');
        expect(state.error).to.be.null;
      });
  });

  it('should put the account data in the state for existing accounts', () => {
    const { store, backend, mockIpc } = setupBackendAndStore();
    mockIpc.getAccount = () => new Promise(r => r('123'));
    mockIpc.getAccountData = () => new Promise(r => r({
      expiry: '2001-01-01T00:00:00Z',
    }));

    return backend.autologin()
      .then( () => {
        const state = store.getState().account;
        expect(state.status).to.equal('ok');
        expect(state.accountToken).to.equal('123');
        expect(state.expiry).to.equal('2001-01-01T00:00:00Z');
      });
  });

  it('should redirect to /connect for existing accounts', () => {
    const { store, backend, mockIpc } = setupBackendAndMockStore();

    mockIpc.getAccount = () => new Promise(r => r('123'));
    mockIpc.getAccountData = () => new Promise(r => r({
      expiry: '2001-01-01T00:00:00Z',
    }));

    return backend.autologin()
      .then( () => {
        expect(getLocation(store)).to.equal('/connect');
      });
  });
});
