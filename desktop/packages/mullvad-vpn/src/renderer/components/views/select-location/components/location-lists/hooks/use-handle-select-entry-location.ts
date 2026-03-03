import React from 'react';

import { type RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectEntryLocation() {
  const { setLocationType } = useSelectLocationViewContext();
  const { selectEntryRelayLocation } = useRelayLocations();

  const handleSelectEntryLocation = React.useCallback(
    async (entryLocation: RelayLocation) => {
      setLocationType(LocationType.exit);
      await selectEntryRelayLocation(entryLocation);
    },
    [selectEntryRelayLocation, setLocationType],
  );

  return handleSelectEntryLocation;
}
