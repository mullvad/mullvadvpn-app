// @flow
import { handleActions } from 'redux-actions';
import actions from '../actions/connect';

import type { ReduxAction } from '../store';
import type { ConnectionState } from '../enums';

export type ConnectReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  serverAddress: ?string,
  clientIp: ?string
};

const initialState: ConnectReduxState = {
  status: 'disconnected',
  isOnline: true,
  serverAddress: null,
  clientIp: null
};

export default handleActions({
  [actions.connectionChange.toString()]: (state: ConnectReduxState, action: ReduxAction<$Shape<ConnectReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
