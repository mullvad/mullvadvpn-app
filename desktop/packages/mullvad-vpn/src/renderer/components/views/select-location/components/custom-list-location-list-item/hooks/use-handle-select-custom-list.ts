import React from 'react';

import type { RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';

export function useHandleSelectCustomList() {
  const { handleSelect } = useLocationListsContext();

  return React.useCallback(
    async (location: RelayLocation) => {
      // Only the geographical part should be sent to the daemon when setting a location.
      if ('country' in location) {
        const { customList: _, ...geographicalLocation } = location;
        await handleSelect(geographicalLocation);
      } else {
        await handleSelect(location);
      }
    },
    [handleSelect],
  );
}
