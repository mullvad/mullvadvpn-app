import assert from 'assert';
import { createAction } from 'redux-actions';
import { LoginState } from '../constants';

const loginChange = createAction('USER_LOGIN_CHANGE');

const requestLogin = (backend, account) => {
  return async (dispatch, getState) => {
    try {
      dispatch(loginChange({ account: account, status: LoginState.connecting }));
      
      await backend.login(account);

      dispatch(loginChange({ status: LoginState.ok }));
    } catch(e) {
      dispatch(loginChange({ status: LoginState.failed, error: e }));
    }
  };
};

export default {
  requestLogin, 
  loginChange 
};

