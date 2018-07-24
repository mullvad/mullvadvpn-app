// @flow

import { Clipboard } from 'reactxp';
import type { AccountToken } from '../../lib/daemon-rpc';
import type { ReduxThunk } from '../store';

const copyAccountToken = (): ReduxThunk => {
  return (_, getState) => {
    const accountToken = getState().account.accountToken;
    if (accountToken) {
      Clipboard.setText(accountToken);
    }
  };
};

type StartLoginAction = {
  type: 'START_LOGIN',
  accountToken?: AccountToken,
};

type LoginSuccessfulAction = {
  type: 'LOGIN_SUCCESSFUL',
  expiry?: string,
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

function startLogin(accountToken?: AccountToken): StartLoginAction {
  return {
    type: 'START_LOGIN',
    accountToken: accountToken,
  };
}

function loginSuccessful(expiry: string): LoginSuccessfulAction {
  return {
    type: 'LOGIN_SUCCESSFUL',
    expiry,
  };
}

function loginFailed(error: Error): LoginFailedAction {
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

function resetLoginError(): ResetLoginErrorAction {
  return {
    type: 'RESET_LOGIN_ERROR',
  };
}

function updateAccountToken(token: AccountToken): UpdateAccountTokenAction {
  return {
    type: 'UPDATE_ACCOUNT_TOKEN',
    token: token,
  };
}

function updateAccountHistory(accountHistory: Array<AccountToken>): UpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory: accountHistory,
  };
}

function updateAccountExpiry(expiry: string): UpdateAccountExpiryAction {
  return {
    type: 'UPDATE_ACCOUNT_EXPIRY',
    expiry,
  };
}

export default {
  copyAccountToken,
  startLogin,
  loginSuccessful,
  loginFailed,
  loggedOut,
  autoLoginFailed,
  resetLoginError,
  updateAccountToken,
  updateAccountHistory,
  updateAccountExpiry,
};
