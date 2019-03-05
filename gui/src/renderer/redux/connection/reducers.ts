import { Ip, TunnelStateTransition } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

export interface IConnectionReduxState {
  status: TunnelStateTransition;
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
      return { ...state, ...action.newLocation };

    case 'UPDATE_BLOCK_STATE':
      return { ...state, isBlocked: action.isBlocked };

    case 'CONNECTING':
      return {
        ...state,
        status: { state: 'connecting', details: action.tunnelEndpoint },
      };

    case 'CONNECTED':
      return {
        ...state,
        status: { state: 'connected', details: action.tunnelEndpoint },
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

    case 'BLOCKED':
      return {
        ...state,
        status: { state: 'blocked', details: action.reason },
      };

    default:
      return state;
  }
}
