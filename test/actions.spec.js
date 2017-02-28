import { expect } from 'chai';
import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import Backend from '../app/lib/backend';
import userActions from '../app/actions/user';
import mapBackendEventsToReduxActions from '../app/lib/backend-redux-actions';
import { LoginState, ConnectionState, defaultServer } from '../app/constants';

const middlewares = [ thunk ];
const mockStore = configureMockStore(middlewares);
const mockState = () => {
  return {
    user: {
      account: null,
      status: LoginState.none,
      error: null
    },
    connect: {
      status: ConnectionState.disconnected,
      serverAddress: null,
      clientIp: null
    },
    settings: {
      autoSecure: false,
      preferredServer: defaultServer
    }
  };
};

const filterIpUpdateActions = (actions) => {
  return actions.filter((action) => {
    return !(action.type === 'CONNECTION_CHANGE' && action.payload.clientIp);
  });
};

describe('actions', function() {
  this.timeout(5000);

  it('should login', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'connecting', error: null, account: '111123456789' } },
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'ok', error: undefined } }
    ];

    const backend = new Backend();
    const store = mockStore(mockState());

    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.login, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    store.dispatch(userActions.login(backend, '111123456789'));
  });

  it('should logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: null, status: 'none', error: null } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      }
    });

    const backend = new Backend();
    const store = mockStore(state);

    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.logout, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(userActions.logout(backend));
  });

});
