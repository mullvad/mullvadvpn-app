// @flow

import { Clipboard } from 'reactxp';
import type { AccountToken } from '../../lib/daemon-rpc';
import type { ReduxThunk } from '../store';

type StartLoginAction = {
  type: 'START_LOGIN',
  accountToken: AccountToken,
};

type LoginSuccessfulAction = {
  type: 'LOGIN_SUCCESSFUL',
};

type LoginFailedAction = {
  type: 'LOGIN_FAILED',
  error: Error,
};

type LoggedOutAction = {
  type: 'LOGGED_OUT',
};

type ResetLoginErrorAction = {
  type: 'RESET_LOGIN_ERROR',
};

type UpdateAccountTokenAction = {
  type: 'UPDATE_ACCOUNT_TOKEN',
  token: AccountToken,
};

type UpdateAccountHistoryAction = {
  type: 'UPDATE_ACCOUNT_HISTORY',
  accountHistory: Array<AccountToken>,
};

type UpdateAccountExpiryAction = {
  type: 'UPDATE_ACCOUNT_EXPIRY',
  expiry: string,
};

export type AccountAction =
  | StartLoginAction
  | LoginSuccessfulAction
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

function loginSuccessful(): LoginSuccessfulAction {
  return {
    type: 'LOGIN_SUCCESSFUL',
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

function copyAccountToken(): ReduxThunk {
  return (_, getState) => {
    const accountToken = getState().account.accountToken;
    if (accountToken) {
      Clipboard.setText(accountToken);
    }
  };
}

export default {
  startLogin,
  loginSuccessful,
  loginFailed,
  loggedOut,
  resetLoginError,
  updateAccountToken,
  updateAccountHistory,
  updateAccountExpiry,
  copyAccountToken,
};
