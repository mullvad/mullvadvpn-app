import { expect } from 'chai';
import userReducer from '../app/reducers/user';
import connectReducer from '../app/reducers/connect';
import settingsReducer from '../app/reducers/settings';
import userActions from '../app/actions/user';
import connectActions from '../app/actions/connect';
import settingsActions from '../app/actions/settings';
import { LoginState, ConnectionState, defaultServer } from '../app/constants';

describe('reducers', () => {

  it('should handle USER_LOGIN_CHANGE', () => {
    const action = userActions.loginChange({
      account: '1111',
      status: LoginState.failed,
      error: new Error('Something went wrong')
    });
    const test = Object.assign({}, action.payload);
    expect(userReducer({}, action)).to.deep.equal(test);
  });

  it('should handle CONNECTION_CHANGE', () => {
    const action = connectActions.connectionChange({ 
      status: ConnectionState.connected,
      serverAddress: '2.1.1.2',
      clientIp: '2.1.1.1'
    });
    const test = Object.assign({}, action.payload);
    expect(connectReducer({}, action)).to.deep.equal(test);
  });

  it('should handle SETTINGS_CHANGE', () => {
    const action = settingsActions.updateSettings({ 
      autoSecure: true,
      preferredServer: defaultServer
    });
    const test = Object.assign({}, action.payload);
    expect(settingsReducer({}, action)).to.deep.equal(test);
  });

});
