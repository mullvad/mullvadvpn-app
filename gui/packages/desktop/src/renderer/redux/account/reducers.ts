import { ReduxAction } from '../store';
import { AccountToken } from '../../../shared/daemon-rpc-types';

export type LoginState = 'none' | 'logging in' | 'failed' | 'ok';
export type AccountReduxState = {
  accountToken?: AccountToken;
  accountHistory: Array<AccountToken>;
  expiry?: string; // ISO8601
  status: LoginState;
  error?: Error;
};

const initialState: AccountReduxState = {
  accountToken: undefined,
  accountHistory: [],
  expiry: undefined,
  status: 'none',
  error: undefined,
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
          error: undefined,
        },
      };
    case 'LOGGED_IN':
      return {
        ...state,
        ...{
          status: 'ok',
          error: undefined,
        },
      };
    case 'LOGIN_FAILED':
      return {
        ...state,
        ...{
          status: 'failed',
          accountToken: undefined,
          error: action.error,
        },
      };
    case 'LOGGED_OUT':
      return {
        ...state,
        ...{
          status: 'none',
          accountToken: undefined,
          expiry: undefined,
          error: undefined,
        },
      };
    case 'RESET_LOGIN_ERROR':
      return {
        ...state,
        ...{
          status: 'none',
          error: undefined,
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
