// @flow

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


export default function(state: ConnectionReduxState = initialState, action: ReduxAction): ConnectionReduxState {

  if (action.type === 'CONNECTION_CHANGE') {
    return { ...state, ...action.newData };
  }

  return state;
}
