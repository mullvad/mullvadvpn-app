// @flow

import { expect } from 'chai';
import connectionReducer from '../app/redux/connection/reducers';
import settingsReducer from '../app/redux/settings/reducers';
import { defaultServer } from '../app/config';

describe('reducers', () => {
  const previousState: any = {};

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
