// @flow

import { expect } from 'chai';
import accountReducer from '../app/redux/account/reducers';
import connectionReducer from '../app/redux/connection/reducers';
import settingsReducer from '../app/redux/settings/reducers';
import { defaultServer } from '../app/config';
import { BackendError } from '../app/lib/backend';

describe('reducers', () => {
  const previousState: any = {};

  it('should handle USER_LOGIN_CHANGE', () => {
    const action = {
      type: 'LOGIN_CHANGE',
      newData: {
        accountNumber: '1111',
        status: 'failed',
        error: new BackendError('INVALID_ACCOUNT')
      }
    };
    const test = Object.assign({}, action.newData);
    expect(accountReducer(previousState, action)).to.deep.equal(test);
  });

  it('should handle CONNECTION_CHANGE', () => {
    const action = {
      type: 'CONNECTION_CHANGE',
      newData: {
        status: 'connected',
        serverAddress: '2.1.1.2',
        clientIp: '2.1.1.1'
      }
    };
    const test = Object.assign({}, action.newData);
    expect(connectionReducer(previousState, action)).to.deep.equal(test);
  });

  it('should handle SETTINGS_UPDATE', () => {
    const action = {
      type: 'UPDATE_SETTINGS',
      newSettings: {
        autoSecure: true,
        preferredServer: defaultServer
      }
    };
    const test = Object.assign({}, action.newSettings);
    expect(settingsReducer(previousState, action)).to.deep.equal(test);
  });

});
