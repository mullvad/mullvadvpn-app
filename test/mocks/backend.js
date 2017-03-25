import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';
import Backend from '../../app/lib/backend';
import Ipc from '../../app/lib/ipc';
import { defaultServer } from '../../app/config';
import { LoginState, ConnectionState } from '../../app/enums';

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

export const mockBackend = (backendData) => {
  return new Backend(mockIpc(backendData));
};

const mockIpc = (backendData) => {
  const ipc = new Ipc();
  ipc.send = (action, data) => {
    return new Promise((resolve, reject) => {

      switch (action) {
      case 'login':
        return resolve(backendData.users[data.accountNumber]);
      case 'logout':
      case 'cancelConnection':
      case 'connect':
      case 'disconnect':
        return resolve();

      case 'getLocation':
        return resolve({});
      case 'getConnectionInfo':
        return resolve({});
      }

      reject('Unknown action: ' + action);
    });
  };
  return ipc;
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
