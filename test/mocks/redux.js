import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import { defaultServer } from '../../app/config';

// fetch is absent in node environment
// this will automatically import it into global scope
import fetch from 'isomorphic-fetch'; // eslint-disable-line no-unused-vars

const middlewares = [ thunk ];
export const mockStore = configureMockStore(middlewares);
export const mockState = () => {
  return {
    account: {
      account: null,
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
