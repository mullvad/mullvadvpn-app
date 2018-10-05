// @flow

import type { AfterDisconnect, BlockReason, Ip } from '../../lib/daemon-rpc';

type ConnectingAction = {
  type: 'CONNECTING',
};

type ConnectedAction = {
  type: 'CONNECTED',
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

function connecting(): ConnectingAction {
  return {
    type: 'CONNECTING',
  };
}

function connected(): ConnectedAction {
  return {
    type: 'CONNECTED',
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
