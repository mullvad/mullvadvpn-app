// @flow

import { Clipboard } from 'reactxp';

import type { Backend } from '../../lib/backend';
import type { ReduxThunk } from '../store';
import type { Coordinate2d } from '../../types';

const connect = (backend: Backend): ReduxThunk => () => backend.connect();
const disconnect = (backend: Backend) => () => backend.disconnect();
const copyIPAddress = (): ReduxThunk => {
  return (_, getState) => {
    const { connection: { clientIp } } = getState();
    if(clientIp) {
      Clipboard.setText(clientIp);
    }
  };
};


type ConnectingAction = {
  type: 'CONNECTING',
};
type ConnectedAction = {
  type: 'CONNECTED',
};
type DisconnectedAction = {
  type: 'DISCONNECTED',
};

type NewPublicIpAction = {
  type: 'NEW_PUBLIC_IP',
  ip: string,
};

type Location = {
  location: Coordinate2d,
  country: string,
  city: string,
};

type NewLocationAction = {
  type: 'NEW_LOCATION',
  newLocation: Location,
};

type OnlineAction = {
  type: 'ONLINE',
};

type OfflineAction = {
  type: 'OFFLINE',
};

export type ConnectionAction = NewPublicIpAction
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

function newPublicIp(ip: string): NewPublicIpAction {
  return {
    type: 'NEW_PUBLIC_IP',
    ip: ip,
  };
}

function newLocation(newLoc: Location): NewLocationAction {
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


export default { connect, disconnect, copyIPAddress, newPublicIp, newLocation, connecting, connected, disconnected, online, offline };

