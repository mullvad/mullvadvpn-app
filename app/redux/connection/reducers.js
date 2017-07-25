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

  switch (action.type) {
  case 'CONNECTION_CHANGE':
    return { ...state, ...action.newData };
  case 'NEW_PUBLIC_IP':
    return { ...state, ...{ clientIp: action.ip }};
  case 'NEW_LOCATION':
    return { ...state, ...action.newLocation };
  default:
    return state;
  }
}
