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

interface IStartCreateAccount {
  type: 'START_CREATE_ACCOUNT';
}

interface ICreateAccountFailed {
  type: 'CREATE_ACCOUNT_FAILED';
  error: Error;
}

interface IAccountCreated {
  type: 'ACCOUNT_CREATED';
  token: AccountToken;
  expiry: string;
}

interface IUpdateAccountTokenAction {
  type: 'UPDATE_ACCOUNT_TOKEN';
  token: AccountToken;
}

interface IUpdateAccountHistoryAction {
  type: 'UPDATE_ACCOUNT_HISTORY';
  accountHistory?: AccountToken;
}

interface IUpdateAccountExpiryAction {
  type: 'UPDATE_ACCOUNT_EXPIRY';
  expiry?: string;
  previousExpiry?: string;
}

export type AccountAction =
  | IStartLoginAction
  | ILoggedInAction
  | ILoginFailedAction
  | ILoggedOutAction
  | IResetLoginErrorAction
  | IStartCreateAccount
  | ICreateAccountFailed
  | IAccountCreated
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

function startCreateAccount(): IStartCreateAccount {
  return {
    type: 'START_CREATE_ACCOUNT',
  };
}

function createAccountFailed(error: Error): ICreateAccountFailed {
  return {
    type: 'CREATE_ACCOUNT_FAILED',
    error,
  };
}

function accountCreated(token: AccountToken, expiry: string): IAccountCreated {
  return {
    type: 'ACCOUNT_CREATED',
    token,
    expiry,
  };
}

function updateAccountToken(token: AccountToken): IUpdateAccountTokenAction {
  return {
    type: 'UPDATE_ACCOUNT_TOKEN',
    token,
  };
}

function updateAccountHistory(accountHistory?: AccountToken): IUpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory,
  };
}

function updateAccountExpiry(expiry?: string, previousExpiry?: string): IUpdateAccountExpiryAction {
  return {
    type: 'UPDATE_ACCOUNT_EXPIRY',
    expiry,
    previousExpiry,
  };
}

export default {
  startLogin,
  loggedIn,
  loginFailed,
  loggedOut,
  resetLoginError,
  startCreateAccount,
  createAccountFailed,
  accountCreated,
  updateAccountToken,
  updateAccountHistory,
  updateAccountExpiry,
};
