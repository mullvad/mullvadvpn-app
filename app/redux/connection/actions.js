// @flow

import { clipboard } from 'electron';

import type { Backend } from '../../lib/backend';
import type { ConnectionReduxState } from './reducers.js';
import type { ReduxGetState, ReduxDispatch } from '../store';


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


export type ConnectionChangeAction = {
  type: 'CONNECTION_CHANGE',
  newData: $Shape<ConnectionReduxState>,
};

function connectionChange(newData: $Shape<ConnectionReduxState>): ConnectionChangeAction {
  return {
    type: 'CONNECTION_CHANGE',
    newData: newData,
  };
}


export default { connect, disconnect, copyIPAddress, connectionChange };
