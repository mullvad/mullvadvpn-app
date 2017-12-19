// @flow

import type { RelaySettingsRedux } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettingsRedux,
};

export type SettingsAction = UpdateRelayAction;

function updateRelay(relay: RelaySettingsRedux): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

export default { updateRelay };
