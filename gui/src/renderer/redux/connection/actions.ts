import {
  AfterDisconnect,
  BlockReason,
  ILocation,
  ITunnelEndpoint,
} from '../../../shared/daemon-rpc-types';

interface IConnectingAction {
  type: 'CONNECTING';
  tunnelEndpoint?: ITunnelEndpoint;
}

interface IConnectedAction {
  type: 'CONNECTED';
  tunnelEndpoint: ITunnelEndpoint;
}

interface IDisconnectedAction {
  type: 'DISCONNECTED';
}

interface IDisconnectingAction {
  type: 'DISCONNECTING';
  afterDisconnect: AfterDisconnect;
}

interface IBlockedAction {
  type: 'BLOCKED';
  reason: BlockReason;
}

interface INewLocationAction {
  type: 'NEW_LOCATION';
  newLocation: ILocation;
}

export type ConnectionAction =
  | INewLocationAction
  | IConnectingAction
  | IConnectedAction
  | IDisconnectedAction
  | IDisconnectingAction
  | IBlockedAction;

function connecting(tunnelEndpoint?: ITunnelEndpoint): IConnectingAction {
  return {
    type: 'CONNECTING',
    tunnelEndpoint,
  };
}

function connected(tunnelEndpoint: ITunnelEndpoint): IConnectedAction {
  return {
    type: 'CONNECTED',
    tunnelEndpoint,
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

function blocked(reason: BlockReason): IBlockedAction {
  return {
    type: 'BLOCKED',
    reason,
  };
}

function newLocation(location: ILocation): INewLocationAction {
  return {
    type: 'NEW_LOCATION',
    newLocation: location,
  };
}

export default {
  newLocation,
  connecting,
  connected,
  disconnected,
  disconnecting,
  blocked,
};
