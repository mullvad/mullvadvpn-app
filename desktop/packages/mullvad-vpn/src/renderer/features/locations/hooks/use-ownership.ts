import React from 'react';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useOwnership(): {
  entryOwnership: Ownership;
  exitOwnership: Ownership;
  setEntryOwnership: (entryOwnership: Ownership) => Promise<void>;
  setExitOwnership: (exitOwnership: Ownership) => Promise<void>;
} {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const normalRelaySettings = useNormalRelaySettings();

  const exitOwnership = normalRelaySettings?.ownership || Ownership.any;
  const entryOwnership = normalRelaySettings?.wireguard.entryOwnership || Ownership.any;

  const setEntryOwnership = React.useCallback(
    async (entryOwnership: Ownership) => {
      await relaySettingsUpdater((settings) => {
        return {
          ...settings,
          wireguardConstraints: {
            ...settings.wireguardConstraints,
            entryOwnership,
          },
        };
      });
    },
    [relaySettingsUpdater],
  );

  const setExitOwnership = React.useCallback(
    async (exitOwnership: Ownership) => {
      await relaySettingsUpdater((settings) => {
        return {
          ...settings,
          ownership: exitOwnership,
        };
      });
    },
    [relaySettingsUpdater],
  );

  return { entryOwnership, exitOwnership, setEntryOwnership, setExitOwnership };
}
