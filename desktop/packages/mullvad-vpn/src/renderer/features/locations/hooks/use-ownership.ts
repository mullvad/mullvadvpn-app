import React from 'react';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';

export function useOwnership(locationType: LocationType): {
  ownership: Ownership;
  setOwnership: (selectedOwnership: Ownership) => Promise<void>;
} {
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const ownership = useSelector((state) => {
    if ('normal' in state.settings.relaySettings) {
      if (locationType === LocationType.exit) {
        return state.settings.relaySettings.normal.ownership;
      }

      if (locationType === LocationType.entry || locationType === LocationType.entryAutomatic) {
        return state.settings.relaySettings.normal.wireguard.entryOwnership;
      }
    }

    return Ownership.any;
  });

  const setOwnership = React.useCallback(
    async (ownership: Ownership) => {
      await relaySettingsUpdater((settings) => {
        if (locationType === LocationType.exit) {
          return {
            ...settings,
            ownership,
          };
        }

        if (locationType === LocationType.entry || locationType === LocationType.entryAutomatic) {
          return {
            ...settings,
            wireguardConstraints: {
              ...settings.wireguardConstraints,
              entryOwnership: ownership,
            },
          };
        }

        return settings;
      });
    },
    [locationType, relaySettingsUpdater],
  );

  return { ownership, setOwnership };
}
