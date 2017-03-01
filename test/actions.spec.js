import { expect } from 'chai';

import { filterIpUpdateActions, mockBackend, mockState, mockStore } from './support';
import Backend from '../app/lib/backend';
import userActions from '../app/actions/user';
import connectActions from '../app/actions/connect';
import mapBackendEventsToReduxActions from '../app/lib/backend-redux-actions';
import { LoginState, ConnectionState } from '../app/constants';

describe('actions', function() {
  this.timeout(10000);

  it('should login', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'connecting', error: null, account: '111123456789' } },
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'ok', error: null } }
    ];

    const store = mockStore(mockState());
    const backend = mockBackend(store);
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

    const store = mockStore(state);
    const backend = mockBackend(store);
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.logout, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(userActions.logout(backend));
  });

  it('should connect to VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: '1.2.3.4', status: 'connecting', error: null } },
      { type: 'CONNECTION_CHANGE', payload: { status: 'connected', error: null } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      }
    });

    const store = mockStore(state);
    const backend = mockBackend(store);
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.connect, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(connectActions.connect(backend, '1.2.3.4'));
  });

  it('should fail to connect to VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: 'se1.mullvad.net', status: 'connecting', error: null } },
      { type: 'CONNECTION_CHANGE', payload: { status: 'disconnected', error: new Error('Server is unreachable.') } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      }
    });

    const store = mockStore(state);
    const backend = mockBackend(store);
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.connect, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(connectActions.connect(backend, 'se1.mullvad.net'));
  });

  it('should disconnect from VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: null, status: 'disconnected', error: null } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: ConnectionState.connected
      }
    });

    const store = mockStore(state);
    const backend = mockBackend(store);
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.disconnect, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(connectActions.disconnect(backend));
  });

  it('should disconnect from VPN server on logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: null, status: 'none', error: null } },
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: null, status: 'disconnected', error: null } }
    ];

    let state = Object.assign(mockState(), {
      user: {
        account: '1111234567890',
        status: LoginState.ok
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: ConnectionState.connected
      }
    });

    const store = mockStore(state);
    const backend = mockBackend(store);
    mapBackendEventsToReduxActions(backend, store);

    backend.once(Backend.EventType.disconnect, () => {
      const storeActions = filterIpUpdateActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(userActions.logout(backend));
  });

});
