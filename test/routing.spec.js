import { expect } from 'chai';

import { filterMinorActions, mockBackend, mockState, mockStore } from './mocks/backend';
import userActions from '../app/actions/user';
import mapBackendEventsToRouter from '../app/lib/backend-routing';
import { LoginState } from '../app/enums';

describe('routing', function() {
  this.timeout(10000);

  it('should redirect to login screen on logout', () => {
    const expectedActions = [
      { type: '@@router/CALL_HISTORY_METHOD', payload: { method: 'replace', args: [ '/' ] } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      }
    });

    const store = mockStore(state);
    const backend = mockBackend();
    mapBackendEventsToRouter(backend, store);
    
    store.dispatch(userActions.logout(backend));

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
    const backend = mockBackend({
      users: {
        '1': { status: LoginState.none },
      }
    });
    mapBackendEventsToRouter(backend, store);
    
    store.subscribe(() => {
      const storeActions = filterMinorActions(store.getActions());
      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    store.dispatch(userActions.login(backend, '1'));
  });

});
