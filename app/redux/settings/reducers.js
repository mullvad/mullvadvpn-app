// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';

export type RelaySettings = {
    host: string,
    port: number,
    protocol: 'tcp' | 'udp',
};

export type SettingsReduxState = {
  relaySettings: RelaySettings
};

const initialState: SettingsReduxState = {
  relaySettings: {
    host: defaultServer,
    port: 1301,
    protocol: 'udp',
  },
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  if (action.type === 'UPDATE_RELAY') {
    return { ...state,
      relaySettings: {
        ...state.relaySettings,
        ...action.relay,
      },
    };
  }

  return state;
}
