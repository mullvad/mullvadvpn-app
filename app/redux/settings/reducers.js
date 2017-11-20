// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';

export type RelaySettings = {
    host: string,
    port: 'any' | number,
    protocol: 'any' | 'tcp' | 'udp',
};

export type SettingsReduxState = {
  relaySettings: RelaySettings
};

const initialState: SettingsReduxState = {
  relaySettings: {
    host: defaultServer,
    port: 'any',
    protocol: 'any',
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
