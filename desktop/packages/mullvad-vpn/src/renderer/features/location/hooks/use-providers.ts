import React from 'react';

import { providersFromRelays } from '../../../components/views/filter/utils';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import { getActiveProviders } from '../utils';

export function useProviders(): {
  providers: string[];
  activeProviders: string[];
  setProviders: (selectedProviders: string[]) => Promise<void>;
} {
  const relaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const locations = useSelector((state) => state.settings.relayLocations);
  const providerConstraint = relaySettings?.providers ?? [];

  const providers = providersFromRelays(locations);
  const activeProviders = getActiveProviders(providers, providerConstraint);

  const setProviders = React.useCallback(
    async (selectedProviders: string[]) => {
      await relaySettingsUpdater((settings) => {
        // The daemon expects the value to be an empty list if all are selected.
        const providerSettings =
          selectedProviders.length === providers.length ? [] : selectedProviders;
        return {
          ...settings,
          providers: providerSettings,
        };
      });
    },
    [relaySettingsUpdater, providers.length],
  );

  return { providers, activeProviders, setProviders };
}
