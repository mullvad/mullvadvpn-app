import { createAction } from 'redux-actions';
import { replace } from 'react-router-redux';
import { LoginState } from '../constants';

const loginChange = createAction('USER_LOGIN_CHANGE');
const login = (backend, account) => () => backend.login(account);
const logout = (backend) => () => backend.logout();

export default { login, logout, loginChange };
