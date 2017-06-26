// @flow

import { expect } from 'chai';
import accountReducer from '../app/redux/account/reducers';
import connectionReducer from '../app/redux/connection/reducers';
import settingsReducer from '../app/redux/settings/reducers';
import { defaultServer } from '../app/config';

describe('reducers', () => {

  it('should handle USER_LOGIN_CHANGE', () => {
    const action = {
      type: 'USER_LOGIN_CHANGE',
      payload: {
        account: '1111',
        status: 'failed',
        error: new Error('Something went wrong')
      }
    };
    const test = Object.assign({}, action.payload);
    expect(accountReducer({}, action)).to.deep.equal(test);
  });

  it('should handle CONNECTION_CHANGE', () => {
    const action = {
      type: 'CONNECTION_CHANGE',
      payload: {
        status: 'connected',
        serverAddress: '2.1.1.2',
        clientIp: '2.1.1.1'
      }
    };
    const test = Object.assign({}, action.payload);
    expect(connectionReducer({}, action)).to.deep.equal(test);
  });

  it('should handle SETTINGS_UPDATE', () => {
    const action = {
      type: 'SETTINGS_UPDATE',
      payload: {
        autoSecure: true,
        preferredServer: defaultServer
      }
    };
    const test = Object.assign({}, action.payload);
    expect(settingsReducer({}, action)).to.deep.equal(test);
  });

});
