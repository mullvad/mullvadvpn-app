import React from 'react';

import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { setLocationType, setSearchTerm } = useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: AnyLocation) => {
      await selectEntryRelayLocation(entryLocation.details);
      // Scroll and isolated item is reset in the LocationListSlide component
      setLocationType(LocationType.exit);
      setSearchTerm('');
    },
    [selectEntryRelayLocation, setLocationType, setSearchTerm],
  );

  return handleSelectEntryLocation;
}
