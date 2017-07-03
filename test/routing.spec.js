// @flow

import { expect } from 'chai';

import { filterMinorActions, mockState, mockStore } from './mocks/redux';
import accountActions from '../app/redux/account/actions';
import mapBackendEventsToRouter from '../app/lib/backend-routing';
import { Backend } from '../app/lib/backend';
import { newMockIpc } from './mocks/ipc';

describe('routing', function() {
  this.timeout(10000);

  it('should redirect to login screen on logout', () => {
    const expectedActions = [
      { type: '@@router/CALL_HISTORY_METHOD', payload: { method: 'replace', args: [ '/' ] } }
    ];

    let state = Object.assign(mockState(), {
      account: {
        account: '1111234567890',
        status: 'ok'
      }
    });

    const store = mockStore(state);
    const backend = new Backend(newMockIpc());
    mapBackendEventsToRouter(backend, store);

    store.dispatch(accountActions.logout(backend));

    setTimeout(() => {
      const storeActions = filterMinorActions(store.getActions());
      expect(storeActions).deep.equal(expectedActions);
    }, 0);
  });

  it('should redirect to connect screen on login', (done) => {
    const expectedActions = [
      { type: '@@router/CALL_HISTORY_METHOD', payload: { method: 'replace', args: [ '/connect' ] } }
    ];

    const store = mockStore(mockState());
    const backend = new Backend(newMockIpc());
    mapBackendEventsToRouter(backend, store);

    store.subscribe(() => {
      const storeActions = filterMinorActions(store.getActions());
      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    store.dispatch(accountActions.login(backend, '1'));
  });

});
