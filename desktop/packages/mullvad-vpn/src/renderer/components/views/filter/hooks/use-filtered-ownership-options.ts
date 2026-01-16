import React from 'react';

import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { filterLocations } from '../../../../lib/filter-locations';
import { useSelector } from '../../../../redux/store';

// Returns only the ownership options that are compatible with the other filters
export function useFilteredOwnershipOptions(
  providers: string[],
  ownership: Ownership,
): Ownership[] {
  const locations = useSelector((state) => state.settings.relayLocations);

  const availableOwnershipOptions = React.useMemo(() => {
    const relaylistForFilters = filterLocations(locations, ownership, providers);

    const filteredRelayOwnership = relaylistForFilters.flatMap((country) =>
      country.cities.flatMap((city) => city.relays.map((relay) => relay.owned)),
    );

    const ownershipOptions = [Ownership.any];
    if (filteredRelayOwnership.includes(true)) {
      ownershipOptions.push(Ownership.mullvadOwned);
    }
    if (filteredRelayOwnership.includes(false)) {
      ownershipOptions.push(Ownership.rented);
    }

    return ownershipOptions;
  }, [locations, ownership, providers]);

  return availableOwnershipOptions;
}
