// @flow

import type { ReduxAction } from '../store';
import type { BackendError } from '../../lib/backend';

export type LoginState = 'none' | 'logging in' | 'failed' | 'ok';
export type AccountReduxState = {
  accountNumber: ?string,
  paidUntil: ?string, // ISO8601
  status: LoginState,
  error: ?BackendError
};

const initialState: AccountReduxState = {
  accountNumber: null,
  paidUntil: null,
  status: 'none',
  error: null
};

export default function(state: AccountReduxState = initialState, action: ReduxAction): AccountReduxState {

  switch (action.type) {
  case 'LOGIN_CHANGE':
    return { ...state, ...action.newData };
  case 'START_LOGIN':
    return { ...state, ...{
      status: 'logging in',
      accountNumber: action.accountNumber,
      error: null,
    }};
  case 'LOGIN_SUCCESSFUL':
    return { ...state, ...{
      status: 'ok',
      error: null,
      paidUntil: action.paidUntil,
    }};
  }

  return state;
}
