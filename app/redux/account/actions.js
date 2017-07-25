// @flow

import type { Backend } from '../../lib/backend';
import type { AccountReduxState } from './reducers.js';

type StartLoginAction = {
  type: 'START_LOGIN',
  accountNumber: string,
};

type LoginChangeAction = {
  type:'LOGIN_CHANGE',
  newData: $Shape<AccountReduxState>,
};

export type AccountAction = StartLoginAction | LoginChangeAction;

function startLogin(accountNumber: string): StartLoginAction {
  return {
    type: 'START_LOGIN',
    accountNumber: accountNumber,
  };
}

function loginChange(data: $Shape<AccountReduxState>): LoginChangeAction {
  return {
    type: 'LOGIN_CHANGE',
    newData: data,
  };
}

const login = (backend: Backend, account: string) => () => backend.login(account);
const logout = (backend: Backend) => () => backend.logout();

export default { login, logout, loginChange, startLogin };
