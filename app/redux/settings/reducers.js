// @flow

import type { ReduxAction } from '../store';
import type { RelayProtocol, RelayLocation } from '../../lib/ipc-facade';

export type RelaySettingsRedux = {|
    normal: {
      location: 'any' | RelayLocation,
      port: 'any' | number,
      protocol: 'any' | RelayProtocol,
    }
|} | {|
  custom_tunnel_endpoint: {
    host: string,
    port: number,
    protocol: RelayProtocol,
  }
|};

export type SettingsReduxState = {
  relaySettings: RelaySettingsRedux
};

const initialState: SettingsReduxState = {
  relaySettings: {
    normal: {
      location: 'any',
      port: 'any',
      protocol: 'any',
    }
  },
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  if (action.type === 'UPDATE_RELAY') {
    return { ...state,
      relaySettings: action.relay,
    };
  }

  return state;
}
