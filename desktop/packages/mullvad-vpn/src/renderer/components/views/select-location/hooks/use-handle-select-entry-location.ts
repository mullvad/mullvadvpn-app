import React from 'react';

import { type RelayLocation } from '../../../../../shared/daemon-rpc-types';
import { useSelectLocation } from '../../../../features/location/hooks';
import { LocationType } from '../select-location-types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { setLocationType } = useSelectLocationViewContext();
  const { selectEntryLocation } = useSelectLocation();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: RelayLocation) => {
      setLocationType(LocationType.exit);
      await selectEntryLocation(entryLocation);
    },
    [selectEntryLocation, setLocationType],
  );

  return handleSelectEntryLocation;
}
