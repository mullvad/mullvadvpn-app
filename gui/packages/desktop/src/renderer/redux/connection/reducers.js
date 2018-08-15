// @flow

import type { ReduxAction } from '../store';
import type { Ip } from '../../lib/daemon-rpc';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';
export type ConnectionReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  ip: null,
  latitude: null,
  longitude: null,
  country: null,
  city: null,
};

export default function(
  state: ConnectionReduxState = initialState,
  action: ReduxAction,
): ConnectionReduxState {
  switch (action.type) {
    case 'NEW_LOCATION':
      return { ...state, ...action.newLocation };

    case 'CONNECTING':
      return { ...state, ...{ status: 'connecting' } };

    case 'CONNECTED':
      return { ...state, ...{ status: 'connected' } };

    case 'DISCONNECTED':
      return { ...state, ...{ status: 'disconnected' } };

    case 'ONLINE':
      return { ...state, ...{ isOnline: true } };

    case 'OFFLINE':
      return { ...state, ...{ isOnline: false } };

    default:
      return state;
  }
}
