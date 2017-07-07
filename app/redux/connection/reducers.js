// @flow
import { handleActions } from 'redux-actions';
import actions from './actions';

import type { ReduxAction } from '../store';
import type { Coordinate2d } from '../../types';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';
export type ConnectionReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  serverAddress: ?string,
  clientIp: ?string,
  location: ?Coordinate2d,
  country: ?string,
  city: ?string,
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  serverAddress: null,
  clientIp: null,
  location: null,
  country: null,
  city: null,
};

export default handleActions({
  [actions.connectionChange.toString()]: (state: ConnectionReduxState, action: ReduxAction<$Shape<ConnectionReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
