import React from 'react';

import type { GeographicalLocation } from '../../../../../../features/locations/types';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';

export function useHandleSelectLocationInCustomList() {
  const { handleSelect } = useLocationListsContext();

  return React.useCallback(
    async (location: GeographicalLocation) => {
      const locationWithoutCustomList = removeCustomListFromLocationDetails(location);
      await handleSelect(locationWithoutCustomList);
    },
    [handleSelect],
  );
}

function removeCustomListFromLocationDetails<T extends GeographicalLocation>(location: T): T {
  const { customList, ...locationWithoutCustomList } = location.details;
  return {
    ...location,
    details: locationWithoutCustomList,
  };
}
