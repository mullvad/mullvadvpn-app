// @flow

import { expect } from 'chai';
import { filterMinorActions, mockState, mockStore } from './mocks/redux';
import { Backend } from '../app/lib/backend';
import { newMockIpc } from './mocks/ipc';
import accountActions from '../app/redux/account/actions';
import connectionActions from '../app/redux/connection/actions';
import mapBackendEventsToReduxActions from '../app/lib/backend-redux-actions';

describe('actions', function() {
  this.timeout(10000);

  it('should login', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { status: 'connecting', error: null, account: '1'} },
      { type: 'USER_LOGIN_CHANGE', payload: { paidUntil: '2013-01-01T00:00:00.000Z', status: 'ok', error: undefined } }
    ];
    const store = mockStore(mockState());
    const mockIpc = newMockIpc();
    mockIpc.getAccountData = () => {
      return new Promise(r => r({
        paid_until: '2013-01-01T00:00:00.000Z',
      }));
    };

    const backend = new Backend(mockIpc);

    mapBackendEventsToReduxActions(backend, store);

    backend.once('login', () => {
      const storeActions = filterMinorActions(store.getActions());
      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    store.dispatch(accountActions.login(backend, '1'));
  });
  
  it('should logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: '', paidUntil: null, status: 'none', error: null } },
    ];

    const store = mockStore(mockState());
    const mockIpc = newMockIpc();
    const backend = new Backend(mockIpc);
    mapBackendEventsToReduxActions(backend, store);

    backend.once('logout', () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    store.dispatch(accountActions.logout(backend));
  });

  it('should connect to VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: '1.2.3.4', status: 'connecting' } },
      { type: 'CONNECTION_CHANGE', payload: { status: 'connected' } }
    ];

    const store = mockStore(mockState());
    const mockIpc = newMockIpc();
    const backend = new Backend(mockIpc);
    mockIpc.getAccountData = () => {
      return new Promise(r => r({
        paid_until: '2038-01-01T00:00:00.000Z',
      }));
    };
    mapBackendEventsToReduxActions(backend, store);

    backend.once('connect', () => {

      const storeActions = filterMinorActions(store.getActions())
        .filter(action => {
          return action.type === 'CONNECTION_CHANGE' && action.payload.status !== 'disconnected';
        });

      expect(storeActions).deep.equal(expectedActions);
      done();
    });

    backend.once('login', () => {
      store.dispatch(connectionActions.connect(backend, '1.2.3.4'));
    });
    store.dispatch(accountActions.login(backend, '1'));
  });

  it('should disconnect from VPN server', (done) => {
    const expectedActions = [
      { type: 'CONNECTION_CHANGE', payload: { status: 'disconnected', serverAddress: null } }
    ];

    let state = Object.assign(mockState(), {
      account: {
        account: '3333234567890',
        paidUntil: '2038-01-01T00:00:00.000Z',
        status: 'ok'
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: 'connected'
      }
    });

    const store = mockStore(state);
    const backend = new Backend(newMockIpc());
    mapBackendEventsToReduxActions(backend, store);

    backend.once('disconnect', () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(connectionActions.disconnect(backend));
  });

  it('should disconnect from VPN server on logout', (done) => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: { account: '', paidUntil: null, status: 'none', error: null } },
      { type: 'CONNECTION_CHANGE', payload: { serverAddress: null, status: 'disconnected' } }
    ];

    let state = Object.assign(mockState(), {
      account: {
        account: '3333234567890',
        paidUntil: '2038-01-01T00:00:00.000Z',
        status: 'ok'
      },
      connect: {
        serverAddress: '1.2.3.4',
        status: 'connected'
      }
    });

    const store = mockStore(state);
    const backend = new Backend(newMockIpc());
    mapBackendEventsToReduxActions(backend, store);

    backend.once('disconnect', () => {
      const storeActions = filterMinorActions(store.getActions());

      expect(storeActions).deep.equal(expectedActions);
      done();
    });
    
    store.dispatch(accountActions.logout(backend));
  });

});
