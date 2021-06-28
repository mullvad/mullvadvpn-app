import { AccountToken } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

type LoginMethod = 'existing_account' | 'new_account';
export type LoginState =
  | { type: 'none' }
  | { type: 'logging in' | 'ok'; method: LoginMethod }
  | { type: 'failed'; method: LoginMethod; error: Error };
export interface IAccountReduxState {
  accountToken?: AccountToken;
  accountHistory?: AccountToken;
  expiry?: string; // ISO8601
  previousExpiry?: string; // ISO8601
  status: LoginState;
}

const initialState: IAccountReduxState = {
  accountToken: undefined,
  accountHistory: undefined,
  expiry: undefined,
  previousExpiry: undefined,
  status: { type: 'none' },
};

export default function (
  state: IAccountReduxState = initialState,
  action: ReduxAction,
): IAccountReduxState {
  switch (action.type) {
    case 'START_LOGIN':
      return {
        ...state,
        status: { type: 'logging in', method: 'existing_account' },
        accountToken: action.accountToken,
      };
    case 'LOGGED_IN':
      return {
        ...state,
        status: { type: 'ok', method: 'existing_account' },
      };
    case 'LOGIN_FAILED':
      return {
        ...state,
        status: { type: 'failed', method: 'existing_account', error: action.error },
        accountToken: undefined,
      };
    case 'LOGGED_OUT':
      return {
        ...state,
        status: { type: 'none' },
        accountToken: undefined,
        expiry: undefined,
        previousExpiry: undefined,
      };
    case 'RESET_LOGIN_ERROR':
      return {
        ...state,
        status: { type: 'none' },
      };
    case 'START_CREATE_ACCOUNT':
      return {
        ...state,
        status: { type: 'logging in', method: 'new_account' },
      };
    case 'CREATE_ACCOUNT_FAILED':
      return {
        ...state,
        status: { type: 'failed', method: 'new_account', error: action.error },
      };
    case 'ACCOUNT_CREATED':
      return {
        ...state,
        status: { type: 'ok', method: 'new_account' },
        accountToken: action.token,
        expiry: action.expiry,
        previousExpiry: undefined,
      };
    case 'UPDATE_ACCOUNT_TOKEN':
      return {
        ...state,
        accountToken: action.token,
      };
    case 'UPDATE_ACCOUNT_HISTORY':
      return {
        ...state,
        accountHistory: action.accountHistory,
      };
    case 'UPDATE_ACCOUNT_EXPIRY':
      return {
        ...state,
        expiry: action.expiry,
        previousExpiry: action.previousExpiry,
      };
  }

  return state;
}
