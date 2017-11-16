import configureMockStore from 'redux-mock-store';
import thunk from 'redux-thunk';

const middlewares = [ thunk ];
export const mockStore = configureMockStore(middlewares);

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
