import { expect } from 'chai';
import actions from '../../app/actions/user';

describe('actions', () => {

  describe('user', () => {
    
    it('should log in', () => {
      const action = { 
        type: 'USER_LOGIN', 
        payload: { 
          username: 'John Doe', 
          loggedIn: true 
        }
      };
      const payload = Object.assign({}, action.payload);
      expect(actions.login(payload)).to.deep.equal(action);
    });

  });
});
