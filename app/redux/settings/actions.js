// @flow

import type { RelaySettings } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettings,
};

export type SettingsAction = UpdateRelayAction;

function updateRelay(relay: RelaySettings): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

export default { updateRelay };
