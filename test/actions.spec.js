import { expect } from 'chai';
import { filterMinorActions, mockBackend, mockState, mockStore } from './mocks/backend';
import Backend from '../app/lib/backend';
import userActions from '../app/actions/user';
import connectActions from '../app/actions/connect';
import mapBackendEventsToReduxActions from '../app/lib/backend-redux-actions';
import { LoginState, ConnectionState } from '../app/enums';

describe('actions', function() {
  this.timeout(10000);

  it('should login', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'connecting', error: null, account: '1'} },
      { type: 'USER_LOGIN_CHANGE', payload: { paidUntil: '2013-01-01T00:00:00.000Z', status: 'ok', error: undefined } }
    ];
    const store = mockStore(mockState());
    const backend = mockBackend({
      users: {
        1: {
          paidUntil: '2013-01-01T00:00:00.000Z',
        }}
    });
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.login, () => {
      const storeActions = filterMinorActions(store.getActions());
      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    store.dispatch(userActions.login(backend, '1'));
  });
  
  it('should logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: null, paidUntil: null, status: 'none', error: null } },
    ];

    const store = mockStore(mockState());
    const backend = mockBackend();
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.logout, () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    store.dispatch(userActions.logout(backend));
  });

  it('should connect to VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: '1.2.3.4', status: 'connecting' } },
      { type: 'CONNECTION_CHANGE', payload: { status: 'connected' } }
    ];

    const store = mockStore(mockState());
    const backend = mockBackend({
      users: {
        '1': {
          paidUntil: '2038-01-01T00:00:00.000Z',
          status: LoginState.ok
        }
      }});
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.connect, () => {

      const storeActions = filterMinorActions(store.getActions())
        .filter(action => {
          return action.type === 'CONNECTION_CHANGE' && action.payload.status !== 'disconnected';
        });

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    backend.once(Backend.EventType.login, () => {
      store.dispatch(connectActions.connect(backend, '1.2.3.4'));
    });
    store.dispatch(userActions.login(backend, '1'));
  });

  it('should disconnect from VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { status: 'disconnected', serverAddress: null } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '3333234567890',
        paidUntil: '2038-01-01T00:00:00.000Z',
        status: LoginState.ok
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: ConnectionState.connected
      }
    });

    const store = mockStore(state);
    const backend = mockBackend();
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.disconnect, () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(connectActions.disconnect(backend));
  });

  it('should disconnect from VPN server on logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: null, paidUntil: null, status: 'none', error: null } },
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: null, status: 'disconnected' } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '3333234567890',
        paidUntil: '2038-01-01T00:00:00.000Z',
        status: LoginState.ok
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: ConnectionState.connected
      }
    });

    const store = mockStore(state);
    const backend = mockBackend();
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.disconnect, () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(userActions.logout(backend));
  });

});
