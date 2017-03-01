import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import Backend from '../app/lib/backend';
import { LoginState, ConnectionState, defaultServer } from '../app/constants';

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
    return !(action.type === 'CONNECTION_CHANGE' && action.payload.clientIp);
  });
};
