// @flow

import type { ReduxAction } from '../store';
import type { AccountToken } from '../../lib/daemon-rpc';

export type LoginState = 'none' | 'logging in' | 'failed' | 'ok';
export type AccountReduxState = {
  accountToken: ?AccountToken,
  accountHistory: Array<AccountToken>,
  expiry: ?string, // ISO8601
  status: LoginState,
  error: ?Error,
};

const initialState: AccountReduxState = {
  accountToken: null,
  accountHistory: [],
  expiry: null,
  status: 'none',
  error: null,
};

export default function(
  state: AccountReduxState = initialState,
  action: ReduxAction,
): AccountReduxState {
  switch (action.type) {
    case 'START_LOGIN':
      return {
        ...state,
        ...{
          status: 'logging in',
          accountToken: action.accountToken,
          error: null,
        },
      };
    case 'LOGIN_SUCCESSFUL':
      return {
        ...state,
        ...{
          status: 'ok',
          error: null,
        },
      };
    case 'LOGIN_FAILED':
      return {
        ...state,
        ...{
          status: 'failed',
          accountToken: null,
          error: action.error,
        },
      };
    case 'LOGGED_OUT':
      return {
        ...state,
        ...{
          status: 'none',
          accountToken: null,
          expiry: null,
          error: null,
        },
      };
    case 'RESET_LOGIN_ERROR':
      return {
        ...state,
        ...{
          status: 'none',
          error: null,
        },
      };
    case 'UPDATE_ACCOUNT_TOKEN':
      return {
        ...state,
        ...{
          accountToken: action.token,
        },
      };
    case 'UPDATE_ACCOUNT_HISTORY':
      return {
        ...state,
        ...{
          accountHistory: action.accountHistory,
        },
      };
    case 'UPDATE_ACCOUNT_EXPIRY':
      return {
        ...state,
        ...{
          expiry: action.expiry,
        },
      };
  }

  return state;
}
