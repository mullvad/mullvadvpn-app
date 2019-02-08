import { AccountToken } from '../../../shared/daemon-rpc-types';

interface IStartLoginAction {
  type: 'START_LOGIN';
  accountToken: AccountToken;
}

interface ILoggedInAction {
  type: 'LOGGED_IN';
}

interface ILoginFailedAction {
  type: 'LOGIN_FAILED';
  error: Error;
}

interface ILoggedOutAction {
  type: 'LOGGED_OUT';
}

interface IResetLoginErrorAction {
  type: 'RESET_LOGIN_ERROR';
}

interface IUpdateAccountTokenAction {
  type: 'UPDATE_ACCOUNT_TOKEN';
  token: AccountToken;
}

interface IUpdateAccountHistoryAction {
  type: 'UPDATE_ACCOUNT_HISTORY';
  accountHistory: AccountToken[];
}

interface IUpdateAccountExpiryAction {
  type: 'UPDATE_ACCOUNT_EXPIRY';
  expiry: string;
}

export type AccountAction =
  | IStartLoginAction
  | ILoggedInAction
  | ILoginFailedAction
  | ILoggedOutAction
  | IResetLoginErrorAction
  | IUpdateAccountTokenAction
  | IUpdateAccountHistoryAction
  | IUpdateAccountExpiryAction;

function startLogin(accountToken: AccountToken): IStartLoginAction {
  return {
    type: 'START_LOGIN',
    accountToken,
  };
}

function loggedIn(): ILoggedInAction {
  return {
    type: 'LOGGED_IN',
  };
}

function loginFailed(error: Error): ILoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error,
  };
}

function loggedOut(): ILoggedOutAction {
  return {
    type: 'LOGGED_OUT',
  };
}

function resetLoginError(): IResetLoginErrorAction {
  return {
    type: 'RESET_LOGIN_ERROR',
  };
}

function updateAccountToken(token: AccountToken): IUpdateAccountTokenAction {
  return {
    type: 'UPDATE_ACCOUNT_TOKEN',
    token,
  };
}

function updateAccountHistory(accountHistory: AccountToken[]): IUpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory,
  };
}

function updateAccountExpiry(expiry: string): IUpdateAccountExpiryAction {
  return {
    type: 'UPDATE_ACCOUNT_EXPIRY',
    expiry,
  };
}

export default {
  startLogin,
  loggedIn,
  loginFailed,
  loggedOut,
  resetLoginError,
  updateAccountToken,
  updateAccountHistory,
  updateAccountExpiry,
};
