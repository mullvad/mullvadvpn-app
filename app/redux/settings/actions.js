// @flow

import type { RelaySettingsRedux, RelayLocationRedux } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettingsRedux,
};

export type UpdateRelayLocationsAction = {
  type: 'UPDATE_RELAY_LOCATIONS',
  relayLocations: Array<RelayLocationRedux>,
};

export type UpdateAllowLanAction = {
  type: 'UPDATE_ALLOW_LAN',
  allowLan: boolean,
};

export type SettingsAction = UpdateRelayAction | UpdateRelayLocationsAction | UpdateAllowLanAction;

function updateRelay(relay: RelaySettingsRedux): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

function updateRelayLocations(
  relayLocations: Array<RelayLocationRedux>,
): UpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations: relayLocations,
  };
}

function updateAllowLan(allowLan: boolean): UpdateAllowLanAction {
  return {
    type: 'UPDATE_ALLOW_LAN',
    allowLan: allowLan,
  };
}

export default { updateRelay, updateRelayLocations, updateAllowLan };
