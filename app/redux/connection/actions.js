// @flow

import { Clipboard } from 'reactxp';
import type { ReduxThunk } from '../store';
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

function copyIPAddress(): ReduxThunk {
  return (_, getState) => {
    const ip = getState().connection.ip;
    if (ip) {
      Clipboard.setText(ip);
    }
  };
}

export default {
  newLocation,
  connecting,
  connected,
  disconnected,
  online,
  offline,
  copyIPAddress,
};
