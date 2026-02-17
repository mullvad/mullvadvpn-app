import React from 'react';

import type { RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';

export function useHandleSelectCustomList() {
  const { handleSelect } = useLocationListsContext();

  return React.useCallback(
    async (value: RelayLocation) => {
      const location = { ...value };
      if ('country' in location) {
        // Only the geographical part should be sent to the daemon when setting a location.
        delete location.customList;
      }
      await handleSelect(location);
    },
    [handleSelect],
  );
}
