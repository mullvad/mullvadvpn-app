// @flow

import type { ReduxAction } from '../store';
import type { TunnelStateTransition, Ip } from '../../lib/daemon-rpc';

export type ConnectionReduxState = {
  status: TunnelStateTransition,
  isOnline: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
};

const initialState: ConnectionReduxState = {
  status: { state: 'disconnected' },
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
      return { ...state, status: { state: 'connecting', details: action.tunnelEndpoint } };

    case 'CONNECTED':
      return { ...state, status: { state: 'connected', details: action.tunnelEndpoint } };

    case 'DISCONNECTED':
      return { ...state, status: { state: 'disconnected' } };

    case 'DISCONNECTING':
      return { ...state, status: { state: 'disconnecting', details: action.afterDisconnect } };

    case 'BLOCKED':
      return { ...state, status: { state: 'blocked', details: action.reason } };

    case 'ONLINE':
      return { ...state, isOnline: true };

    case 'OFFLINE':
      return { ...state, isOnline: false };

    default:
      return state;
  }
}
