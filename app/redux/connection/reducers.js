// @flow
import { handleActions } from 'redux-actions';
import actions from './actions';

import type { ReduxAction } from '../store';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';
export type ConnectionReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  serverAddress: ?string,
  clientIp: ?string
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  serverAddress: null,
  clientIp: null
};

export default handleActions({
  [actions.connectionChange.toString()]: (state: ConnectionReduxState, action: ReduxAction<$Shape<ConnectionReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
