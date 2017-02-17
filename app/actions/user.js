import { createAction } from 'redux-actions';
import { replace } from 'react-router-redux';
import { LoginState } from '../constants';

const loginChange = createAction('USER_LOGIN_CHANGE');

const login = (backend, account) => {
  return async (dispatch) => {
    try {
      dispatch(loginChange({ status: LoginState.connecting, account }));
      
      await backend.login(account);

      // report success to login screen
      dispatch(loginChange({ status: LoginState.ok }));

      // show connection screen after delay
      setTimeout(() => dispatch(replace('/connect')), 1000);
    } catch(e) {
      dispatch(loginChange({ status: LoginState.failed, error: e }));
    }
  };
};

const logout = (backend) => {
  return async (dispatch) => {
    try {
      await backend.logout();
    } catch(e) {
      console.log(`Failed to log out: ${e.message}`);
    }

    // reset login information
    dispatch(loginChange({ status: LoginState.none, account: '', error: null }));

    // redirect user to /
    dispatch(replace('/'));
  };
};

export default {
  login,
  logout,
  loginChange
};

