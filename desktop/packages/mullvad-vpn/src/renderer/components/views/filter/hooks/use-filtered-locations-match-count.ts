import React from 'react';

import { RelaySelectorPartitions } from '../../../../../shared/relay-selector-rpc-types';
import { useAppContext } from '../../../../context';
import { useQuery } from '../../../../lib/hooks';
import { useSelector } from '../../../../redux/store';
import { convertSettingsToRelaySelectorQueries } from '../../../../utils';
import { useFilterViewContext } from '../FilterViewContext';

export function useFilteredLocationsMatchCount() {
  const { selectedOwnership, selectedProviders, locationType } = useFilterViewContext();
  const { getRelayPartitions } = useAppContext();

  const settings = useSelector((state) => state.settings);
  const settingsWithSelections = React.useMemo(() => {
    if ('normal' in settings.relaySettings) {
      if (locationType === 'entry') {
        return {
          ...settings,
          relaySettings: {
            ...settings.relaySettings,
            normal: {
              ...settings.relaySettings.normal,
              wireguard: {
                ...settings.relaySettings.normal.wireguard,
                entryOwnership: selectedOwnership,
                entryProviders: selectedProviders,
              },
            },
          },
        };
      }

      if (locationType === 'exit') {
        return {
          ...settings,
          relaySettings: {
            ...settings.relaySettings,
            normal: {
              ...settings.relaySettings.normal,
              ownership: selectedOwnership,
              providers: selectedProviders,
            },
          },
        };
      }
    }
    return settings;
  }, [settings, locationType, selectedOwnership, selectedProviders]);

  const queryRelayPartitions = React.useCallback(async () => {
    const relaySelectorQueries = convertSettingsToRelaySelectorQueries(settingsWithSelections);
    if (relaySelectorQueries) {
      const relaySelectorQuery = relaySelectorQueries.find(
        (query) => query.context === locationType,
      );
      if (relaySelectorQuery) {
        return getRelayPartitions(relaySelectorQuery.predicate);
      }
    }

    return Promise.resolve(undefined);
  }, [getRelayPartitions, locationType, settingsWithSelections]);
  const queryKey = [...selectedProviders, selectedOwnership.toString()];

  const { isLoading, isFetching, data } = useQuery<RelaySelectorPartitions>({
    queryFn: queryRelayPartitions,
    queryKey,
  });

  const matchingServers = data?.matches.length ?? 0;

  return {
    isLoading,
    isFetching,
    matchingServers,
  };
}
