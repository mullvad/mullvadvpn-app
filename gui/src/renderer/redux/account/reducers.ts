import { AccountToken } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

export type LoginState = 'none' | 'logging in' | 'failed' | 'ok';
export type CreateAccountStatus = 'none' | 'creating account' | 'failed' | 'ok';
export interface IAccountReduxState {
  accountToken?: AccountToken;
  accountHistory: AccountToken[];
  expiry?: string; // ISO8601
  status: LoginState;
  createAccountStatus: CreateAccountStatus;
  error?: Error;
}

const initialState: IAccountReduxState = {
  accountToken: undefined,
  accountHistory: [],
  expiry: undefined,
  status: 'none',
  createAccountStatus: 'none',
  error: undefined,
};

export default function(
  state: IAccountReduxState = initialState,
  action: ReduxAction,
): IAccountReduxState {
  switch (action.type) {
    case 'START_LOGIN':
      return {
        ...state,
        ...{
          status: 'logging in',
          accountToken: action.accountToken,
          createAccountStatus: 'none',
          error: undefined,
        },
      };
    case 'LOGGED_IN':
      return {
        ...state,
        ...{
          status: 'ok',
          createAccountStatus: state.createAccountStatus === 'creating account' ? 'ok' : 'none',
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
          createAccountStatus: 'none',
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
          createAccountStatus: 'none',
          error: undefined,
        },
      };
    case 'START_CREATE_ACCOUNT':
      return {
        ...state,
        ...{
          createAccountStatus: 'creating account',
          status: 'none',
          accountToken: undefined,
          error: undefined,
        },
      };
    case 'CREATE_ACCOUNT_FAILED':
      return {
        ...state,
        ...{
          createAccountStatus: 'failed',
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
