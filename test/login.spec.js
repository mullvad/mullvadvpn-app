// @flow

import { expect } from 'chai';
import { setupBackendAndStore } from './helpers/ipc-helpers';
import { IpcChain } from './helpers/IpcChain';
import accountActions from '../app/redux/account/actions';

describe('Logging in', () => {

  it('should validate the account number and then set it in the backend', (done) => {
    const { store, mockIpc, backend } = setupBackendAndStore();

    const chain = new IpcChain(mockIpc, done);
    chain.addRequiredStep('getAccountData')
      .withInputValidation((an) => {
        expect(an).to.equal('123');
      })
      .done();

    chain.addRequiredStep('setAccount')
      .withInputValidation((an) => {
        expect(an).to.equal('123');
      })
      .done();

    const action: any = accountActions.login(backend, '123');
    store.dispatch(action);
  });
});

