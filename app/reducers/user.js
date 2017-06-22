// @flow
import { handleActions } from 'redux-actions';
import actions from '../actions/user';

import type { ReduxAction } from '../store';
import type { LoginState } from '../enums';
import type { BackendError } from '../lib/backend';

export type UserReduxState = {
  account: ?string,
  paidUntil: ?string, // ISO8601
  location: Array<number>,
  country: ?string,
  city: ?string,
  status: LoginState,
  error: ?BackendError
};

const initialState: UserReduxState = {
  account: null,
  paidUntil: null,
  location: [0, 0],
  country: null,
  city: null,
  status: 'none',
  error: null
};

export default handleActions({
  [actions.loginChange.toString()]: (state: UserReduxState, action: ReduxAction<$Shape<UserReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
