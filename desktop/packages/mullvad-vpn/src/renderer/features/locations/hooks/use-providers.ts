import React from 'react';

import { providersFromRelays } from '../../../components/views/filter/utils';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import { getActiveProviders } from '../utils';

export function useProviders(): {
  providers: string[];
  entryProviders: string[];
  exitProviders: string[];
  setEntryProviders: (selectedEntryProviders: string[]) => Promise<void>;
  setExitProviders: (selectedExitProviders: string[]) => Promise<void>;
} {
  const relaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const locations = useSelector((state) => state.settings.relayLocations);
  const providers = providersFromRelays(locations);

  const entryProviderConstraint = relaySettings?.wireguard.entryProviders ?? [];
  const entryProviders = getActiveProviders(providers, entryProviderConstraint);

  const exitProviderConstraint = relaySettings?.providers ?? [];
  const exitProviders = getActiveProviders(providers, exitProviderConstraint);

  const setEntryProviders = React.useCallback(
    async (selectedEntryProviders: string[]) => {
      await relaySettingsUpdater((settings) => {
        // The daemon expects the value to be an empty list if all are selected.
        const entryProviders =
          selectedEntryProviders.length === providers.length ? [] : selectedEntryProviders;

        return {
          ...settings,
          wireguardConstraints: {
            ...settings.wireguardConstraints,
            entryProviders: entryProviders,
          },
        };
      });
    },
    [relaySettingsUpdater, providers.length],
  );

  const setExitProviders = React.useCallback(
    async (selectedExitProviders: string[]) => {
      await relaySettingsUpdater((settings) => {
        // The daemon expects the value to be an empty list if all are selected.
        const exitProviders =
          selectedExitProviders.length === providers.length ? [] : selectedExitProviders;

        return {
          ...settings,
          providers: exitProviders,
        };
      });
    },
    [relaySettingsUpdater, providers.length],
  );

  return { providers, exitProviders, entryProviders, setEntryProviders, setExitProviders };
}
