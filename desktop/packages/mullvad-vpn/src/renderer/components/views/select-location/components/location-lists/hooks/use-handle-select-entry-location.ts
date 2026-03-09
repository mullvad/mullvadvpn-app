import React from 'react';

import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { setLocationType } = useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: AnyLocation) => {
      setLocationType(LocationType.exit);
      await selectEntryRelayLocation(entryLocation.details);
    },
    [selectEntryRelayLocation, setLocationType],
  );

  return handleSelectEntryLocation;
}
