import React from 'react';

import { providersFromRelays } from '../../../components/views/filter/utils';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';
import { getActiveProviders } from '../utils';

export function useProviders(locationType: LocationType): {
  providers: string[];
  activeProviders: string[];
  setProviders: (selectedProviders: string[]) => Promise<void>;
} {
  const relaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const locations = useSelector((state) => state.settings.relayLocations);

  const getProvidersConstraint = () => {
    if (locationType === LocationType.exit) {
      return relaySettings?.providers ?? [];
    }

    if (locationType === LocationType.entry || locationType === LocationType.entryAutomatic) {
      return relaySettings?.wireguard?.entryProviders ?? [];
    }

    return [];
  };
  const providerConstraint = getProvidersConstraint();

  const providers = providersFromRelays(locations);
  const activeProviders = getActiveProviders(providers, providerConstraint);

  const setProviders = React.useCallback(
    async (selectedProviders: string[]) => {
      await relaySettingsUpdater((settings) => {
        // The daemon expects the value to be an empty list if all are selected.
        const providerSettings =
          selectedProviders.length === providers.length ? [] : selectedProviders;

        if (locationType === LocationType.exit) {
          return {
            ...settings,
            providers: providerSettings,
          };
        }

        if (locationType === LocationType.entry || locationType === LocationType.entryAutomatic) {
          return {
            ...settings,
            wireguardConstraints: {
              ...settings.wireguardConstraints,
              entryProviders: providerSettings,
            },
          };
        }

        return settings;
      });
    },
    [relaySettingsUpdater, providers.length, locationType],
  );

  return { providers, activeProviders, setProviders };
}
