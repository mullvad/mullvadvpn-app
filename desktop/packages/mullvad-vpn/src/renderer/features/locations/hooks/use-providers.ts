import React from 'react';

import { providersFromRelays } from '../../../components/views/filter/utils';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSettingsRelayLocationsFiltered } from '../../../redux/hooks';
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

  const relays = locations.flatMap((location) =>
    location.cities.flatMap((city) => city.relays.map((relay) => relay)),
  );

  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  // TODO: Map relay locations filtered to their corresponding location

  const entryProviderConstraint = relayLocationsFiltered.entry.matches
    .map((relayMatch) => relayMatch.relay.hostname)
    .reduce((activeProviders, hostname) => {
      const relay = relays.find((relay) => relay.hostname === hostname);
      if (relay) {
        if (!activeProviders.includes(relay.provider)) {
          return [...activeProviders, relay.provider];
        }
      }

      return activeProviders;
    }, relaySettings?.wireguard.entryProviders ?? []);
  const entryProviders = entryProviderConstraint;

  const exitProviderConstraint = relayLocationsFiltered.exit.matches
    .map((relayMatch) => relayMatch.relay.hostname)
    .reduce((activeProviders, hostname) => {
      const relay = relays.find((relay) => relay.hostname === hostname);
      if (relay) {
        if (!activeProviders.includes(relay.provider)) {
          return [...activeProviders, relay.provider];
        }
      }

      return activeProviders;
    }, relaySettings?.providers ?? []);
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
