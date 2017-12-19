// @flow

import type { RelaySettingsRedux, RelayLocationRedux } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettingsRedux,
};

export type UpdateRelayLocationsAction = {
  type: 'UPDATE_RELAY_LOCATIONS',
  relayLocations: Array<RelayLocationRedux>
}

export type SettingsAction = UpdateRelayAction | UpdateRelayLocationsAction;

function updateRelay(relay: RelaySettingsRedux): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

function updateRelayLocations(relayLocations: Array<RelayLocationRedux>): UpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations: relayLocations,
  };
}

export default { updateRelay, updateRelayLocations };
