// @flow

import { clipboard } from 'electron';

import type { Backend } from '../../lib/backend';
import type { ConnectionReduxState } from './reducers.js';
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

type ConnectionChangeAction = {
  type: 'CONNECTION_CHANGE',
  newData: $Shape<ConnectionReduxState>,
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

export type ConnectionAction = ConnectionChangeAction
                                | NewPublicIpAction
                                | NewLocationAction
                                | ConnectingAction
                                | ConnectedAction
                                | DisconnectedAction;

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

function connectionChange(newData: $Shape<ConnectionReduxState>): ConnectionChangeAction {
  return {
    type: 'CONNECTION_CHANGE',
    newData: newData,
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


export default { connect, disconnect, copyIPAddress, connectionChange, newPublicIp, newLocation, connectingTo, connecting, connected, disconnected };

