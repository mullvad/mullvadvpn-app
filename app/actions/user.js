import { createAction } from 'redux-actions';
import { replace } from 'react-router-redux';
import { LoginState } from '../constants';

const loginChange = createAction('USER_LOGIN_CHANGE');

const login = (backend, account) => {
  return (dispatch) => {
    backend.login(account).then(() => {
      // show connection screen after delay
      setTimeout(() => dispatch(replace('/connect')), 1000);
    });
  };
};

const logout = (backend) => {
  return (dispatch) => {
    return backend.logout().then(() => {
      dispatch(replace('/'));
    });
  };
};

export default { login, logout, loginChange };
