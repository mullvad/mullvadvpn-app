// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';
import type { RelayConstraints } from '../../lib/ipc-facade';

export type SettingsReduxState = {
  relayConstraints: RelayConstraints,
};

const initialState: SettingsReduxState = {
  relayConstraints: {
    host: { only: defaultServer },
    tunnel: { openvpn: {
      port: 'any',
      protocol: 'any',
    }},
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
