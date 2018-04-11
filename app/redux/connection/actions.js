// @flow

import { Clipboard } from 'reactxp';

import type { Backend, BackendError } from '../../lib/backend';
import type { ReduxThunk } from '../store';
import type { Ip } from '../../lib/ipc-facade';

const connect = (backend: Backend): ReduxThunk => () => backend.connect();
const disconnect = (backend: Backend) => () => backend.disconnect();
const copyIPAddress = (): ReduxThunk => {
  return (_, getState) => {
    const ip = getState().connection.ip;
    if(ip) {
      Clipboard.setText(ip);
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

type AuthFailedAction = {
  type: 'AUTH_FAILED',
  cause: ?BackendError,
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

export type ConnectionAction = NewLocationAction
                                | ConnectingAction
                                | ConnectedAction
                                | DisconnectedAction
                                | AuthFailedAction
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

function authFailed(cause: ?BackendError): AuthFailedAction {
  return {
    type: 'AUTH_FAILED',
    cause,
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


export default { connect, disconnect, copyIPAddress, newLocation, connecting, connected, disconnected, authFailed, online, offline };

