import {
  AfterDisconnect,
  ErrorState,
  ILocation,
  ITunnelStateRelayInfo,
} from '../../../shared/daemon-rpc-types';

interface IConnectingAction {
  type: 'CONNECTING';
  details?: ITunnelStateRelayInfo;
}

interface IConnectedAction {
  type: 'CONNECTED';
  details: ITunnelStateRelayInfo;
}

interface IDisconnectedAction {
  type: 'DISCONNECTED';
}

interface IDisconnectingAction {
  type: 'DISCONNECTING';
  afterDisconnect: AfterDisconnect;
}

interface IBlockedAction {
  type: 'TUNNEL_ERROR';
  errorState: ErrorState;
}

interface INewLocationAction {
  type: 'NEW_LOCATION';
  newLocation: Partial<ILocation>;
}

interface IUpdateBlockStateAction {
  type: 'UPDATE_BLOCK_STATE';
  isBlocked: boolean;
}

export type ConnectionAction =
  | INewLocationAction
  | IConnectingAction
  | IConnectedAction
  | IDisconnectedAction
  | IDisconnectingAction
  | IBlockedAction
  | IUpdateBlockStateAction;

function connecting(details?: ITunnelStateRelayInfo): IConnectingAction {
  return {
    type: 'CONNECTING',
    details,
  };
}

function connected(details: ITunnelStateRelayInfo): IConnectedAction {
  return {
    type: 'CONNECTED',
    details,
  };
}

function disconnected(): IDisconnectedAction {
  return {
    type: 'DISCONNECTED',
  };
}

function disconnecting(afterDisconnect: AfterDisconnect): IDisconnectingAction {
  return {
    type: 'DISCONNECTING',
    afterDisconnect,
  };
}

function blocked(errorState: ErrorState): IBlockedAction {
  return {
    type: 'TUNNEL_ERROR',
    errorState,
  };
}

function newLocation(location: Partial<ILocation>): INewLocationAction {
  return {
    type: 'NEW_LOCATION',
    newLocation: location,
  };
}

function updateBlockState(isBlocked: boolean): IUpdateBlockStateAction {
  return {
    type: 'UPDATE_BLOCK_STATE',
    isBlocked,
  };
}

export default {
  newLocation,
  updateBlockState,
  connecting,
  connected,
  disconnected,
  disconnecting,
  blocked,
};
