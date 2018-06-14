// @flow

import { expect } from 'chai';
import {
  setupBackendAndStore,
  setupBackendAndMockStore,
  checkNextTick,
  getLocation,
  failFast,
  check,
} from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import accountActions from '../app/redux/account/actions';

describe('Logging in', () => {
  it('should validate the account number and then set it in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc);
    chain.require('getAccountData').withInputValidation((an) => {
      expect(an).to.equal('123');
    });

    chain.require('setAccount').withInputValidation((an) => {
      expect(an).to.equal('123');
    });

    chain.onSuccessOrFailure(done);

    store.dispatch(accountActions.login(backend, '123'));
  });

  it('should put the account data in the state', () => {
    const { store, backend, mockIpc } = setupBackendAndStore();
    mockIpc.getAccountData = () =>
      new Promise((r) =>
        r({
          expiry: '2001-01-01T00:00:00Z',
        }),
      );

    return backend.login('123').then(() => {
      const state = store.getState().account;
      expect(state.status).to.equal('ok');
      expect(state.accountToken).to.equal('123');
      expect(state.expiry).to.equal('2001-01-01T00:00:00Z');
    });
  });

  it('should indicate failure for non-existing accounts', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    mockIpc.getAccountData = (_num) =>
      new Promise((_, reject) => {
        reject('NO SUCH ACCOUNT');
      });

    store.dispatch(accountActions.login(backend, '123'));

    checkNextTick(() => {
      const state = store.getState().account;
      expect(state.status).to.equal('failed');
      expect(state.error).to.not.be.null;
    }, done);
  });

  it('should redirect to /connect after 1s after successful login', (done) => {
    const { store, backend } = setupBackendAndMockStore();

    store.dispatch(accountActions.login(backend, '123'));

    setTimeout(() => {
      failFast(() => {
        expect(getLocation(store)).not.to.equal('/connect');
      }, done);
    }, 100);

    setTimeout(() => {
      check(() => {
        expect(getLocation(store)).to.equal('/connect');
      }, done);
    }, 1100);
  });
});
