import { expect } from 'chai';
import reducer from '../../app/reducers/user';
import actions from '../../app/actions/user';
import { LoginState } from '../../app/constants';

describe('reducers', () => {

  describe('user', () => {    
    
    it('should handle USER_LOGIN_CHANGE', () => {
      const action = { 
        type: actions.loginChange.toString(),
        payload: {
          account: '1111',
          status: LoginState.failed,
          error: new Error('Something went wrong')
        }
      };
      const test = Object.assign({}, action.payload);
      expect(reducer({}, action)).to.deep.equal(test);
    });
    
  });

});
