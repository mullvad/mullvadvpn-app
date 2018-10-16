// @flow

import type { AfterDisconnect, BlockReason, TunnelEndpoint } from '../../lib/daemon-rpc';

type ConnectingAction = {
  type: 'CONNECTING',
  tunnelEndpoint: ?TunnelEndpoint,
};

type ConnectedAction = {
  type: 'CONNECTED',
  tunnelEndpoint: TunnelEndpoint,
};

type DisconnectedAction = {
  type: 'DISCONNECTED',
};

type DisconnectingAction = {
  type: 'DISCONNECTING',
  afterDisconnect: AfterDisconnect,
};

type BlockedAction = {
  type: 'BLOCKED',
  reason: BlockReason,
};

type NewLocationAction = {
  type: 'NEW_LOCATION',
  newLocation: {
    country: string,
    city: ?string,
    latitude: number,
    longitude: number,
    mullvadExitIp: boolean,
    hostname: ?string,
  },
};

type OnlineAction = {
  type: 'ONLINE',
};

type OfflineAction = {
  type: 'OFFLINE',
};

export type ConnectionAction =
  | NewLocationAction
  | ConnectingAction
  | ConnectedAction
  | DisconnectedAction
  | DisconnectingAction
  | BlockedAction
  | OnlineAction
  | OfflineAction;

function connecting(tunnelEndpoint: ?TunnelEndpoint): ConnectingAction {
  return {
    type: 'CONNECTING',
    tunnelEndpoint,
  };
}

function connected(tunnelEndpoint: TunnelEndpoint): ConnectedAction {
  return {
    type: 'CONNECTED',
    tunnelEndpoint,
  };
}

function disconnected(): DisconnectedAction {
  return {
    type: 'DISCONNECTED',
  };
}

function disconnecting(afterDisconnect: AfterDisconnect): DisconnectingAction {
  return {
    type: 'DISCONNECTING',
    afterDisconnect,
  };
}

function blocked(reason: BlockReason): BlockedAction {
  return {
    type: 'BLOCKED',
    reason,
  };
}

function newLocation(newLoc: $PropertyType<NewLocationAction, 'newLocation'>): NewLocationAction {
  return {
    type: 'NEW_LOCATION',
    newLocation: newLoc,
  };
}

function online(): OnlineAction {
  return {
    type: 'ONLINE',
  };
}

function offline(): OfflineAction {
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
