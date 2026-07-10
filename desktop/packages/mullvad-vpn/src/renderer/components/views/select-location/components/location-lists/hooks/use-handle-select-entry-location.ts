import React, { startTransition } from 'react';

import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../../features/locations/types';
import { waitForAnimations } from '../../../../../../lib/utils';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { entryLocationListsContainerRef, setLocationType, searchTerm, setSearchTerm } =
    useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: AnyLocation) => {
      if (!searchTerm) {
        setLocationType(LocationType.exit);
        await selectEntryRelayLocation(entryLocation.details);
      } else {
        // If the user selects a location from a search, we can't immediately switch
        // to show the `exit` locations as the view contents would jump around
        // too much and confuse the user. To avoid this we just update the selected
        // entry, which will cause an animation to mark it in the location lists
        // as selected, and when that animation has finished we can switch to show
        // `exit` locations.
        startTransition(async () => {
          await selectEntryRelayLocation(entryLocation.details);
          await waitForAnimations(entryLocationListsContainerRef.current);
          setSearchTerm('');
          setLocationType(LocationType.exit);
        });
      }
    },
    [
      entryLocationListsContainerRef,
      searchTerm,
      selectEntryRelayLocation,
      setLocationType,
      setSearchTerm,
    ],
  );

  return handleSelectEntryLocation;
}
