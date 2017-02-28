import { expect } from 'chai';
import userActions from '../app/actions/user';
import connectActions from '../app/actions/connect';
import settingsActions from '../app/actions/settings';
import { LoginState, ConnectionState, defaultServer } from '../app/constants';

describe('actions', () => {

  it('should create action for USER_LOGIN_CHANGE', () => {
    const test = { 
      type: userActions.loginChange.toString(), 
      payload: { 
        account: '1111',
        status: LoginState.failed,
        error: new Error('Something went wrong')
      }
    };
    const payload = Object.assign({}, test.payload);
    expect(userActions.loginChange(payload)).to.deep.equal(test);
  });

  it('should create action for CONNECTION_CHANGE', () => {
    const test = { 
      type: connectActions.connectionChange.toString(), 
      payload: { 
        status: ConnectionState.connected,
        serverAddress: '2.1.1.2',
        clientIp: '2.1.1.1'
      }
    };
    const payload = Object.assign({}, test.payload);
    expect(connectActions.connectionChange(payload)).to.deep.equal(test);
  });

  it('should create action for SETTINGS_UPDATE', () => {
    const test = { 
      type: settingsActions.updateSettings.toString(), 
      payload: { 
        autoSecure: true,
        preferredServer: defaultServer
      }
    };
    const payload = Object.assign({}, test.payload);
    expect(settingsActions.updateSettings(payload)).to.deep.equal(test);
  });

});
