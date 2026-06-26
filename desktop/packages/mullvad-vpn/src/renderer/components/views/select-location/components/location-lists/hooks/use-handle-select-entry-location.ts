import React from 'react';

import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { setLocationType, searchTerm } = useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: AnyLocation) => {
      if (!searchTerm) {
        setLocationType(LocationType.exit);
      }
      await selectEntryRelayLocation(entryLocation.details);
    },
    [searchTerm, selectEntryRelayLocation, setLocationType],
  );

  return handleSelectEntryLocation;
}
