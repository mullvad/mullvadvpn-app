import React from 'react';

import type { RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useHandleSelectEntryLocation, useHandleSelectExitLocation } from '../../../hooks';
import { useCustomListListContext } from '../CustomListLocationListContext';

export function useHandleSelectCustomList() {
  const handleSelectExitRelay = useHandleSelectExitLocation();
  const handleSelectEntryRelay = useHandleSelectEntryLocation();
  const { locationSelection } = useCustomListListContext();

  return React.useCallback(
    async (value: RelayLocation) => {
      const location = { ...value };
      if ('country' in location) {
        // Only the geographical part should be sent to the daemon when setting a location.
        delete location.customList;
      }
      if (locationSelection === 'entry') {
        await handleSelectEntryRelay(location);
      } else {
        await handleSelectExitRelay(location);
      }
    },
    [locationSelection, handleSelectEntryRelay, handleSelectExitRelay],
  );
}
