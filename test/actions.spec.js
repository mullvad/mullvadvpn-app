import { expect } from 'chai';
import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import Backend from '../app/lib/backend';
import userActions from '../app/actions/user';
import { LoginState, ConnectionState, defaultServer } from '../app/constants';

const middlewares = [ thunk ];
const mockStore = configureMockStore(middlewares);

describe('actions', () => {

  it('should create action for USER_LOGIN_CHANGE', () => {
    const expectedActions = [
      { type: 'USER_LOGIN_CHANGE', payload: {} }
    ];

    const backend = new Backend();
    const store = mockStore({});
    
    // @TODO: Figure out how to test actions in event based system
    // store.dispatch(userActions.login(backend, '111123456789'));
  });

});
