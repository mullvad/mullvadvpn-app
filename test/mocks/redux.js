import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import { defaultServer } from '../../app/config';

const middlewares = [ thunk ];
export const mockStore = configureMockStore(middlewares);
export const mockState = () => {
  return {
    account: {
      accountNumber: null,
      status: 'none',
      error: null
    },
    connection: {
      status: 'disconnected',
      serverAddress: null,
      clientIp: null
    },
    settings: {
      autoSecure: false,
      preferredServer: defaultServer
    }
  };
};

export const filterMinorActions = (actions) => {
  return actions.filter((action) => {
    if(action.type === 'CONNECTION_CHANGE' && action.payload.clientIp) {
      return false;
    }

    if(action.type === 'CONNECTION_CHANGE' && action.payload.isOnline) {
      return false;
    }

    if(action.type === 'USER_LOGIN_CHANGE' && action.payload.city) {
      return false;
    }

    return true;
  });
};
