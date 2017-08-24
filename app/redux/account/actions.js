// @flow

import type { Backend, BackendError } from '../../lib/backend';

type StartLoginAction = {
  type: 'START_LOGIN',
  accountNumber?: string,
};
type LoginSuccessfulAction = {
  type: 'LOGIN_SUCCESSFUL',
  expiry: string,
};
type LoginFailedAction = {
  type: 'LOGIN_FAILED',
  error: BackendError,
};

type LoggedOutAction = {
  type: 'LOGGED_OUT',
};

export type AccountAction = StartLoginAction
                            | LoginSuccessfulAction
                            | LoginFailedAction
                            | LoggedOutAction;

function startLogin(accountNumber?: string): StartLoginAction {
  return {
    type: 'START_LOGIN',
    accountNumber: accountNumber,
  };
}

function loginSuccessful(expiry: string): LoginSuccessfulAction {
  return {
    type: 'LOGIN_SUCCESSFUL',
    expiry: expiry,
  };
}

function loginFailed(error: BackendError): LoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error: error,
  };
}

function loggedOut(): LoggedOutAction {
  return {
    type: 'LOGGED_OUT',
  };
}

function autoLoginFailed(): LoggedOutAction {
  return loggedOut();
}

const login = (backend: Backend, account: string) => () => backend.login(account);
const logout = (backend: Backend) => () => backend.logout();

export default { login, logout, startLogin, loginSuccessful, loginFailed, loggedOut, autoLoginFailed };
