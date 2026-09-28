import React from 'react';

import { useProviders } from '../../../../features/locations/hooks';
import { useRelaySettingsUpdater } from '../../../../lib/constraint-updater';
import { useHistory } from '../../../../lib/history';
import { useFilterViewContext } from '../FilterViewContext';

// Applies the changes by sending them to the daemon.
export function useHandleApplyFilter() {
  const { locationType } = useFilterViewContext();
  const { providers } = useProviders();
  const history = useHistory();
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const { availableProviders, selectedProviders, selectedOwnership } = useFilterViewContext();

  return React.useCallback(async () => {
    const appliedProviders =
      selectedProviders.length === providers.length
        ? [] // The daemon expects the value to be an empty list if all are selected.
        : selectedProviders.filter((provider) => availableProviders.includes(provider));

    await relaySettingsUpdater((settings) => {
      if (locationType === 'entry') {
        settings.wireguardConstraints.entryProviders = appliedProviders;
        settings.wireguardConstraints.entryOwnership = selectedOwnership;
      } else if (locationType === 'exit') {
        settings.providers = appliedProviders;
        settings.ownership = selectedOwnership;
      }

      return settings;
    });
    history.pop();
  }, [
    selectedProviders,
    providers.length,
    relaySettingsUpdater,
    history,
    availableProviders,
    locationType,
    selectedOwnership,
  ]);
}
