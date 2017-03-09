import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import Backend from '../app/lib/backend';
import { defaultServer } from '../app/config';
import { LoginState, ConnectionState } from '../app/enums';

// fetch is absent in node environment
// this will automatically import it into global scope
import fetch from 'isomorphic-fetch'; // eslint-disable-line no-unused-vars

const middlewares = [ thunk ];
export const mockStore = configureMockStore(middlewares);
export const mockState = () => {
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

export const mockBackend = (store) => {
  const backend = new Backend();

  // patch backend
  backend.syncWithReduxStore(store);

  return backend;
};

export const filterIpUpdateActions = (actions) => {
  return actions.filter((action) => {
    if(action.type === 'CONNECTION_CHANGE' && action.payload.clientIp) {
      return false;
    }
    
    if(action.type === 'USER_LOGIN_CHANGE' && action.payload.city) {
      return false;
    }

    return true;
  });
};
