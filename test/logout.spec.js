// @flow

import { expect } from 'chai';
import {
  setupBackendAndStore,
  setupBackendAndMockStore,
  getLocation,
  checkNextTick,
  failFastNextTick,
} from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import accountActions from '../app/redux/account/actions';

describe('logging out', () => {
  it('should set the account to null and then disconnect', (done) => {
    const { mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc);
    chain.require('setAccount').withInputValidation((num) => {
      expect(num).to.be.null;
    });
    chain.require('disconnect');
    chain.onSuccessOrFailure(done);

    backend.logout();
  });

  it('should remove the account number from the store', (done) => {
    const { store, backend, mockIpc } = setupBackendAndStore();
    mockIpc.getAccountData = () =>
      new Promise((r) =>
        r({
          expiry: '2001-01-01T00:00:00.000Z',
        }),
      );
    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);

    const expectedLogoutState = {
      status: 'none',
      accountToken: null,
      expiry: null,
      error: null,
    };

    failFastNextTick(() => {
      let state = store.getState().account;
      expect(state).not.to.include(expectedLogoutState);

      backend.logout();

      checkNextTick(() => {
        state = store.getState().account;
        expect(state).to.include(expectedLogoutState);
      }, done);
    }, done);
  });

  it('should redirect to / on logout', (done) => {
    const { store, backend } = setupBackendAndMockStore();

    backend.logout();

    checkNextTick(() => {
      expect(getLocation(store)).to.equal('/');
    }, done);
  });
});
