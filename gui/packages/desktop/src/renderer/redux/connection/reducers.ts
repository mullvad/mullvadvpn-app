import { Ip, TunnelStateTransition } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

export interface IConnectionReduxState {
  status: TunnelStateTransition;
  isOnline: boolean;
  isBlocked: boolean;
  ip?: Ip;
  hostname?: string;
  latitude?: number;
  longitude?: number;
  country?: string;
  city?: string;
}

const initialState: IConnectionReduxState = {
  status: { state: 'disconnected' },
  isOnline: true,
  isBlocked: false,
  ip: undefined,
  hostname: undefined,
  latitude: undefined,
  longitude: undefined,
  country: undefined,
  city: undefined,
};

export default function(
  state: IConnectionReduxState = initialState,
  action: ReduxAction,
): IConnectionReduxState {
  switch (action.type) {
    case 'NEW_LOCATION':
      const { hostname, latitude, longitude, city, country } = action.newLocation;
      return { ...state, hostname, latitude, longitude, city, country };

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
        isBlocked: action.reason.reason !== 'set_firewall_policy_error',
      };

    case 'ONLINE':
      return { ...state, isOnline: true };

    case 'OFFLINE':
      return { ...state, isOnline: false };

    default:
      return state;
  }
}
