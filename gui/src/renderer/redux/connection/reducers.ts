import { Ip, TunnelState } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

export interface IConnectionReduxState {
  status: TunnelState;
  isBlocked: boolean;
  ipv4?: Ip;
  ipv6?: Ip;
  hostname?: string;
  bridgeHostname?: string;
  entryHostname?: string;
  latitude?: number;
  longitude?: number;
  country?: string;
  city?: string;
}

const initialState: IConnectionReduxState = {
  status: { state: 'disconnected' },
  isBlocked: false,
  ipv4: undefined,
  ipv6: undefined,
  hostname: undefined,
  bridgeHostname: undefined,
  entryHostname: undefined,
  latitude: undefined,
  longitude: undefined,
  country: undefined,
  city: undefined,
};

export default function (
  state: IConnectionReduxState = initialState,
  action: ReduxAction,
): IConnectionReduxState {
  switch (action.type) {
    case 'NEW_LOCATION':
      return {
        ...state,
        ipv4: action.newLocation.ipv4,
        ipv6: action.newLocation.ipv6,
        country: action.newLocation.country,
        city: action.newLocation.city,
        latitude: action.newLocation.latitude,
        longitude: action.newLocation.longitude,
        hostname: action.newLocation.hostname,
        bridgeHostname: action.newLocation.bridgeHostname,
        entryHostname: action.newLocation.entryHostname,
      };

    case 'UPDATE_BLOCK_STATE':
      return { ...state, isBlocked: action.isBlocked };

    case 'CONNECTING':
      return {
        ...state,
        status: { state: 'connecting', details: action.details },
      };

    case 'CONNECTED':
      return {
        ...state,
        status: { state: 'connected', details: action.details },
      };

    case 'DISCONNECTED':
      return {
        ...state,
        status: { state: 'disconnected' },
      };

    case 'DISCONNECTING':
      return {
        ...state,
        status: { state: 'disconnecting', details: action.afterDisconnect },
      };

    case 'TUNNEL_ERROR':
      return {
        ...state,
        status: {
          state: 'error',
          details: action.errorState,
        },
      };

    default:
      return state;
  }
}
