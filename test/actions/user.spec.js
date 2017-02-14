import { expect } from 'chai';
import actions from '../../app/actions/user';
import { LoginState } from '../../app/constants';

describe('actions', () => {

  describe('user', () => {
    
    it('should create action for USER_LOGIN_CHANGE', () => {
      const test = { 
        type: actions.loginChange.toString(), 
        payload: { 
          account: '1111',
          status: LoginState.failed,
          error: new Error('Something went wrong')
        }
      };
      const payload = Object.assign({}, test.payload);
      expect(actions.loginChange(payload)).to.deep.equal(test);
    });

  });
});
