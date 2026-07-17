import React from 'react';

import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../../features/locations/types';
import { waitForAnimations } from '../../../../../../lib/utils';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { viewRef, setLocationType, searchTerm, setSearchTerm, setIsolatedItem } =
    useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    (entryLocation: AnyLocation) => {
      if (!searchTerm) {
        React.startTransition(async () => {
          setLocationType(LocationType.exit);
          await waitForAnimations(viewRef.current);

          React.startTransition(async () => {
            await selectEntryRelayLocation(entryLocation.details);
          });
        });
      } else {
        // If the user selects a location from a search, we can't immediately switch
        // to show the `exit` locations as the view contents would jump around
        // too much and confuse the user. To avoid this we just update the selected
        // entry, which will cause an animation to mark it in the location lists
        // as selected, and when that animation has finished we can switch to show
        // `exit` locations and then we can expand the .
        React.startTransition(async () => {
          await selectEntryRelayLocation(entryLocation.details);
          setSearchTerm('');

          React.startTransition(() => {
            setLocationType(LocationType.exit);
            setIsolatedItem(undefined);
          });
        });
      }
    },
    [
      viewRef,
      searchTerm,
      selectEntryRelayLocation,
      setIsolatedItem,
      setLocationType,
      setSearchTerm,
    ],
  );

  return handleSelectEntryLocation;
}
