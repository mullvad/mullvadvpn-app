// @flow

import type { Backend, BackendError } from '../../lib/backend';
import type { AccountReduxState } from './reducers.js';

type StartLoginAction = {
  type: 'START_LOGIN',
  accountNumber: string,
};
type LoginSuccessfulAction = {
  type: 'LOGIN_SUCCESSFUL',
  paidUntil: string,
};
type LoginFailedAction = {
  type: 'LOGIN_FAILED',
  error: BackendError,
};

type LoginChangeAction = {
  type:'LOGIN_CHANGE',
  newData: $Shape<AccountReduxState>,
};

export type AccountAction = StartLoginAction
                            | LoginSuccessfulAction
                            | LoginFailedAction
                            | LoginChangeAction;

function startLogin(accountNumber: string): StartLoginAction {
  return {
    type: 'START_LOGIN',
    accountNumber: accountNumber,
  };
}

function loginSuccessful(paidUntil: string): LoginSuccessfulAction {
  return {
    type: 'LOGIN_SUCCESSFUL',
    paidUntil: paidUntil,
  };
}

function loginFailed(error: BackendError): LoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error: error,
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

export default { login, logout, loginChange, startLogin, loginSuccessful, loginFailed };
