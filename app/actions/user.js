import { createAction } from 'redux-actions';
import { replace } from 'react-router-redux';
import { LoginState } from '../constants';

const loginChange = createAction('USER_LOGIN_CHANGE');

const login = (backend, account) => {
  return async (dispatch) => {
    try {
      dispatch(loginChange({ 
        account: account, 
        status: LoginState.connecting 
      }));
      
      await backend.login(account);

      dispatch(loginChange({ 
        status: LoginState.ok 
      }));
    } catch(e) {
      dispatch(loginChange({ 
        status: LoginState.failed, 
        error: e 
      }));
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
    dispatch(loginChange({ 
      status: LoginState.none, 
      account: '', 
      error: undefined 
    }));

    // redirect user to /
    dispatch(replace('/'));
  };
};

export default {
  login,
  logout,
  loginChange
};

