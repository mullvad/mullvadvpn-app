import { AfterDisconnect, BlockReason, ITunnelEndpoint } from '../../../shared/daemon-rpc-types';

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
  newLocation: {
    country: string;
    city?: string;
    latitude: number;
    longitude: number;
    mullvadExitIp: boolean;
    hostname?: string;
  };
}

interface IOnlineAction {
  type: 'ONLINE';
}

interface IOfflineAction {
  type: 'OFFLINE';
}

export type ConnectionAction =
  | INewLocationAction
  | IConnectingAction
  | IConnectedAction
  | IDisconnectedAction
  | IDisconnectingAction
  | IBlockedAction
  | IOnlineAction
  | IOfflineAction;

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

function newLocation(location: INewLocationAction['newLocation']): INewLocationAction {
  return {
    type: 'NEW_LOCATION',
    newLocation: location,
  };
}

function online(): IOnlineAction {
  return {
    type: 'ONLINE',
  };
}

function offline(): IOfflineAction {
  return {
    type: 'OFFLINE',
  };
}

export default {
  newLocation,
  connecting,
  connected,
  disconnected,
  disconnecting,
  blocked,
  online,
  offline,
};
