// @flow

import type { ReduxAction } from '../store';

export type DaemonReduxState = {
  isConnected: boolean,
};

const initialState: DaemonReduxState = {
  isConnected: false,
};

export default function(
  state: DaemonReduxState = initialState,
  action: ReduxAction,
): DaemonReduxState {
  switch (action.type) {
    case 'DAEMON_CONNECTED':
      return {
        ...state,
        isConnected: true,
      };

    case 'DAEMON_DISCONNECTED':
      return {
        ...state,
        isConnected: false,
      };

    default:
      return state;
  }
}
