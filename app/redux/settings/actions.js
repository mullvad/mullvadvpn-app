// @flow

import type { RelayConstraints } from '../../lib/ipc-facade';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelayConstraints,
};

export type SettingsAction = UpdateRelayAction;

function updateRelay(relay: RelayConstraints): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

export default { updateRelay };
