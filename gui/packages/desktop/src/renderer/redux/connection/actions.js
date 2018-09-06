// @flow

import type { Ip } from '../../lib/daemon-rpc';

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
};

type BlockedAction = {
  type: 'BLOCKED',
  reason: string,
};

type NewLocationAction = {
  type: 'NEW_LOCATION',
  newLocation: {
    ip: Ip,
    country: string,
    city: ?string,
    latitude: number,
    longitude: number,
    mullvadExitIp: boolean,
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

function disconnecting(): DisconnectingAction {
  return {
    type: 'DISCONNECTING',
  };
}

function blocked(reason: string): BlockedAction {
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
