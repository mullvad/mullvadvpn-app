import { useMemo } from 'react';

import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { filterCountriesByOwnershipAndProviders } from '../../../../features/locations/utils';
import { useSelector } from '../../../../redux/store';
import { providersFromRelays } from '../utils';

// Returns only the providers that are compatible with the other filters
export function useFilteredProviders(providers: string[], ownership: Ownership): string[] {
  const locations = useSelector((state) => state.settings.relayLocations);

  const availableProviders = useMemo(() => {
    const relaylistForFilters = filterCountriesByOwnershipAndProviders(
      locations,
      ownership,
      providers,
    );
    return providersFromRelays(relaylistForFilters);
  }, [locations, ownership, providers]);

  return availableProviders;
}
