// @flow

import type { ReduxAction } from '../store';
import type { Ip } from '../../lib/daemon-rpc';

export type ConnectionState =
  | 'disconnected'
  | 'disconnecting'
  | 'connecting'
  | 'connected'
  | 'blocked';
export type ConnectionReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
  blockReason: ?string,
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  ip: null,
  latitude: null,
  longitude: null,
  country: null,
  city: null,
  blockReason: null,
};

export default function(
  state: ConnectionReduxState = initialState,
  action: ReduxAction,
): ConnectionReduxState {
  switch (action.type) {
    case 'NEW_LOCATION':
      return { ...state, ...action.newLocation };

    case 'CONNECTING':
      return { ...state, ...{ status: 'connecting', blockReason: null } };

    case 'CONNECTED':
      return { ...state, ...{ status: 'connected', blockReason: null } };

    case 'DISCONNECTED':
      return { ...state, ...{ status: 'disconnected', blockReason: null } };

    case 'DISCONNECTING':
      return { ...state, ...{ status: 'disconnecting', blockReason: null } };

    case 'BLOCKED':
      return { ...state, ...{ status: 'blocked', blockReason: action.reason } };

    case 'ONLINE':
      return { ...state, ...{ isOnline: true } };

    case 'OFFLINE':
      return { ...state, ...{ isOnline: false } };

    default:
      return state;
  }
}
