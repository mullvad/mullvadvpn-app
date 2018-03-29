// @flow

import type { BackendError } from '../../lib/backend';
import type { ReduxAction } from '../store';
import type { Ip } from '../../lib/ipc-facade';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';
export type ConnectionReduxState = {
  status: ConnectionState,
  isOnline: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
  authFailureCause: ?BackendError,
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  ip: null,
  latitude: null,
  longitude: null,
  country: null,
  city: null,
  authFailureCause: null,
};


export default function(state: ConnectionReduxState = initialState, action: ReduxAction): ConnectionReduxState {

  switch (action.type) {
  case 'NEW_LOCATION':
    return { ...state, ...action.newLocation };

  case 'CONNECTING':
    return { ...state, ...{ status: 'connecting' }};

  case 'CONNECTED':
    return { ...state, ...{ status: 'connected', authFailureCause: null }};

  case 'DISCONNECTED':
    return { ...state, ...{ status: 'disconnected' }};

  case 'AUTH_FAILED':
    return { ...state, ...{ status: 'disconnected', authFailureCause: action.cause }};

  case 'ONLINE':
    return { ...state, ...{ isOnline: true }};

  case 'OFFLINE':
    return { ...state, ...{ isOnline: false }};

  default:
    return state;
  }
}
