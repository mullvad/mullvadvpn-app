// @flow

import type { RelaySettingsRedux, RelayLocationsRedux } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettingsRedux,
};

export type UpdateRelayLocationsAction = {
  type: 'UPDATE_RELAY_LOCATIONS',
  relayLocations: RelayLocationsRedux
}

export type SettingsAction = UpdateRelayAction | UpdateRelayLocationsAction;

function updateRelay(relay: RelaySettingsRedux): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

function updateRelayLocations(relayLocations: RelayLocationsRedux): UpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations: relayLocations,
  };
}

export default { updateRelay, updateRelayLocations };
