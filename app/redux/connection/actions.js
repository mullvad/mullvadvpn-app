import { clipboard } from 'electron';
import { createAction } from 'redux-actions';

import type { Backend } from '../lib/backend';
import type { ConnectionReduxState } from './reducers.js';
import type { ReduxAction, ReduxGetStateFn, ReduxDispatchFn } from '../store';

export type ConnectionChangeAction = <T: $Shape<ConnectionReduxState>>(state: T) => ReduxAction<T>;

const connectionChange: ConnectionChangeAction = createAction('CONNECTION_CHANGE');
const connect = (backend: Backend, addr: string) => () => backend.connect(addr);
const disconnect = (backend: Backend) => () => backend.disconnect();
const copyIPAddress = () => {
  return (_dispatch: ReduxDispatchFn, getState: ReduxGetStateFn) => {
    const ip: ?string = getState().connect.clientIp;
    if(ip) {
      clipboard.writeText(ip);
    }
  };
};

export default { connect, disconnect, copyIPAddress, connectionChange };
