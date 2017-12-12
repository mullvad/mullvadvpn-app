// @flow

import type { ReduxAction } from '../store';
import type { RelayProtocol, RelayLocation, RelayList } from '../../lib/ipc-facade';

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

export type RelayLocationsRedux = RelayList;

export type SettingsReduxState = {
  relaySettings: RelaySettingsRedux,
  relayLocations: RelayLocationsRedux,
};

const initialState: SettingsReduxState = {
  relaySettings: {
    normal: {
      location: 'any',
      port: 'any',
      protocol: 'any',
    }
  },
  relayLocations: {
    countries: []
  },
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  switch(action.type) {
  case 'UPDATE_RELAY':
    return { ...state,
      relaySettings: action.relay,
    };

  case 'UPDATE_RELAY_LOCATIONS':
    return { ...state,
      relayLocations: action.relayLocations,
    };

  default:
    return state;
  }
}
