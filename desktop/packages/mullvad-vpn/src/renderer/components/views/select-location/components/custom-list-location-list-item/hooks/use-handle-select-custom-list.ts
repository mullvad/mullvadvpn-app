import React from 'react';

import type { AnyLocation } from '../../../../../../features/locations/types';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';

export function useHandleSelectCustomList() {
  const { handleSelect } = useLocationListsContext();

  return React.useCallback(
    async (location: AnyLocation) => {
      if (location.type === 'customList') {
        await handleSelect(location);
      } else {
        const locationWithoutCustomList = removeCustomListFromLocationDetails(location);
        await handleSelect(locationWithoutCustomList);
      }
    },
    [handleSelect],
  );
}

function removeCustomListFromLocationDetails<T extends AnyLocation>(location: T): T {
  const { customList, ...locationWithoutCustomList } = location.details;
  return {
    ...location,
    details: locationWithoutCustomList,
  };
}
