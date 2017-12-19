// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';

export type RelayConstraints = {
    host: string,
    port: 'any' | number,
    protocol: 'any' | 'tcp' | 'udp',
};

export type SettingsReduxState = {
  relayConstraints: RelayConstraints
};

const initialState: SettingsReduxState = {
  relayConstraints: {
    host: defaultServer,
    port: 'any',
    protocol: 'any',
  },
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  if (action.type === 'UPDATE_RELAY') {
    return { ...state,
      relayConstraints: {
        ...state.relayConstraints,
        ...action.relay,
      },
    };
  }

  return state;
}
