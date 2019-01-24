import { AccountToken } from '../../../shared/daemon-rpc-types';

type StartLoginAction = {
  type: 'START_LOGIN';
  accountToken: AccountToken;
};

type LoggedInAction = {
  type: 'LOGGED_IN';
};

type LoginFailedAction = {
  type: 'LOGIN_FAILED';
  error: Error;
};

type LoggedOutAction = {
  type: 'LOGGED_OUT';
};

type ResetLoginErrorAction = {
  type: 'RESET_LOGIN_ERROR';
};

type UpdateAccountTokenAction = {
  type: 'UPDATE_ACCOUNT_TOKEN';
  token: AccountToken;
};

type UpdateAccountHistoryAction = {
  type: 'UPDATE_ACCOUNT_HISTORY';
  accountHistory: Array<AccountToken>;
};

type UpdateAccountExpiryAction = {
  type: 'UPDATE_ACCOUNT_EXPIRY';
  expiry: string;
};

export type AccountAction =
  | StartLoginAction
  | LoggedInAction
  | LoginFailedAction
  | LoggedOutAction
  | ResetLoginErrorAction
  | UpdateAccountTokenAction
  | UpdateAccountHistoryAction
  | UpdateAccountExpiryAction;

function startLogin(accountToken: AccountToken): StartLoginAction {
  return {
    type: 'START_LOGIN',
    accountToken: accountToken,
  };
}

function loggedIn(): LoggedInAction {
  return {
    type: 'LOGGED_IN',
  };
}

function loginFailed(error: Error): LoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error,
  };
}

function loggedOut(): LoggedOutAction {
  return {
    type: 'LOGGED_OUT',
  };
}

function resetLoginError(): ResetLoginErrorAction {
  return {
    type: 'RESET_LOGIN_ERROR',
  };
}

function updateAccountToken(token: AccountToken): UpdateAccountTokenAction {
  return {
    type: 'UPDATE_ACCOUNT_TOKEN',
    token,
  };
}

function updateAccountHistory(accountHistory: Array<AccountToken>): UpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory,
  };
}

function updateAccountExpiry(expiry: string): UpdateAccountExpiryAction {
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
