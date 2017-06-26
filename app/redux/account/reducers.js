// @flow
import { handleActions } from 'redux-actions';
import actions from './actions.js';

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

export default handleActions({
  [actions.loginChange.toString()]: (state: AccountReduxState, action: ReduxAction<$Shape<AccountReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
