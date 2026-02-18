import React from 'react';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useSelector } from '../../../redux/store';

export function useOwnership(): {
  activeOwnership: Ownership;
  setOwnership: (selectedOwnership: Ownership) => Promise<void>;
} {
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const activeOwnership = useSelector((state) =>
    'normal' in state.settings.relaySettings
      ? state.settings.relaySettings.normal.ownership
      : Ownership.any,
  );
  const setOwnership = React.useCallback(
    async (ownership: Ownership) => {
      await relaySettingsUpdater((settings) => {
        return {
          ...settings,
          ownership,
        };
      });
    },
    [relaySettingsUpdater],
  );

  return { activeOwnership, setOwnership };
}
