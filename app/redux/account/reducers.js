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

  if (action.type === 'LOGIN_CHANGE') {
    return { ...state, ...action.newData };
  }

  return state;
}
