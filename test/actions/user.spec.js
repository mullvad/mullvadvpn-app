import chai, { expect } from 'chai';
import spy from 'chai-spies';
import actions from '../../app/actions/user';
import { LoginState } from '../../app/constants';

chai.use(spy);

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

    it('should successfully login', (done) => {
      const actionType = actions.loginChange.toString();
      const account = '1234';
      const getState = () => ({});
      let callCounter = 0;
      const dispatch = chai.spy((action) => {
        callCounter += 1;

        if(callCounter == 2) {
          expect(dispatch).to.have.been.called.with({ 
            type: actionType, 
            payload: { account, status: LoginState.connecting } 
          });

          expect(dispatch).to.have.been.called.with({ 
            type: actionType, 
            payload: { status: LoginState.ok } 
          });

          done();
        }
      });

      const backend = {
        login: () => Promise.resolve()
      };

      const action = actions.login(backend, account);

      action(dispatch, getState);
    });

    it('should fail login', (done) => {
      const actionType = actions.loginChange.toString();
      const account = '1234';
      const getState = () => ({});
      let callCounter = 0;
      const dispatch = chai.spy((action) => {
        callCounter += 1;

        if(callCounter == 2) {
          expect(dispatch).to.have.been.called.with({ 
            type: actionType, 
            payload: { account, status: LoginState.connecting } 
          });

          expect(dispatch).to.have.been.called.with({ 
            type: actionType, 
            payload: { status: LoginState.failed, error: new Error('Failed') } 
          });

          done();
        }
      });

      const backend = {
        login: () => Promise.reject(new Error('Failed'))
      };

      const action = actions.login(backend, account);

      action(dispatch, getState);
    });

  });
});
