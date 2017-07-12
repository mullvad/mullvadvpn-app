// @flow

import { expect } from 'chai';
import { setupBackendAndStore, setupBackendAndMockStore, checkNextTick, getLocation, failFast, check } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import accountActions from '../app/redux/account/actions';

describe('Logging in', () => {

  it('should validate the account number and then set it in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc);
    chain.require('getAccountData')
      .withInputValidation((an) => {
        expect(an).to.equal('123');
      })
      .done();

    chain.require('setAccount')
      .withInputValidation((an) => {
        expect(an).to.equal('123');
      })
      .done();

    chain.onSuccessOrFailure(done);

    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);
  });

  it('should put the account data in the state', (done) => {
    const { store, backend, mockIpc } = setupBackendAndStore();
    mockIpc.getAccountData = () => new Promise(r => r({
      paid_until: '2001-01-01T00:00:00',
    }));

    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);

    checkNextTick( () => {
      const state = store.getState().account;
      expect(state.status).to.equal('ok');
      expect(state.accountNumber).to.equal('123');
      expect(state.paidUntil).to.equal('2001-01-01T00:00:00');
    }, done);
  });

  it('should indicate failure for non-existing accounts', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    mockIpc.getAccountData = (_num) => new Promise((_,reject) => {
      reject('NO SUCH ACCOUNT');
    });


    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);


    checkNextTick(() => {
      const state = store.getState().account;
      expect(state.status).to.equal('failed');
      expect(state.error).to.not.be.null;
    }, done);
  });

  it('should redirect to /connect after 1s after successful login', (done) => {
    const { store, backend } = setupBackendAndMockStore();

    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);


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
