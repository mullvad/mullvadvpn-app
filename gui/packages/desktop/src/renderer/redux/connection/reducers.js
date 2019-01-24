// @flow

import type { ReduxAction } from '../store';
import type { TunnelStateTransition, Ip } from '../../../shared/daemon-rpc-types';

export type ConnectionReduxState = {
  status: TunnelStateTransition,
  isOnline: boolean,
  isBlocked: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
};

const initialState: ConnectionReduxState = {
  status: { state: 'disconnected' },
  isOnline: true,
  isBlocked: false,
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
      return {
        ...state,
        status: { state: 'connecting', details: action.tunnelEndpoint },
        isBlocked: true,
      };

    case 'CONNECTED':
      return {
        ...state,
        status: { state: 'connected', details: action.tunnelEndpoint },
        isBlocked: false,
      };

    case 'DISCONNECTED':
      return { ...state, status: { state: 'disconnected' }, isBlocked: false };

    case 'DISCONNECTING':
      return {
        ...state,
        status: { state: 'disconnecting', details: action.afterDisconnect },
        isBlocked: true,
      };

    case 'BLOCKED':
      return {
        ...state,
        status: { state: 'blocked', details: action.reason },
        isBlocked: action.reason.reason !== 'set_security_policy_error',
      };

    case 'ONLINE':
      return { ...state, isOnline: true };

    case 'OFFLINE':
      return { ...state, isOnline: false };

    default:
      return state;
  }
}
