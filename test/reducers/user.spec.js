import { expect } from 'chai';
import reducer from '../../app/reducers/user';

describe('reducers', () => {

  describe('user', () => {    
    
    it('should handle USER_LOGIN', () => {
      const action = { 
        type: 'USER_LOGIN',
        payload: {
          username: 'John Doe', 
          loggedIn: true 
        }
      };
      const test = Object.assign({}, action.payload);
      expect(reducer({}, action)).to.deep.equal(test);
    });
    
  });

});
