// @flow

import { clipboard } from 'electron';

import type { Backend } from '../../lib/backend';
import type { ReduxGetState, ReduxDispatch } from '../store';
import type { Coordinate2d } from '../../types';


const connect = (backend: Backend, addr: string) => () => backend.connect(addr);
const disconnect = (backend: Backend) => () => backend.disconnect();
const copyIPAddress = () => {
  return (_dispatch: ReduxDispatch, getState: ReduxGetState) => {
    const ip: ?string = getState().connection.clientIp;
    if(ip) {
      clipboard.writeText(ip);
    }
  };
};


type ConnectingAction = {
  type: 'CONNECTING',
  serverAddress?: string,
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

function connectingTo(serverAddress: string): ConnectingAction {
  return {
    type: 'CONNECTING',
    serverAddress: serverAddress,
  };
}

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


export default { connect, disconnect, copyIPAddress, newPublicIp, newLocation, connectingTo, connecting, connected, disconnected, online, offline };

