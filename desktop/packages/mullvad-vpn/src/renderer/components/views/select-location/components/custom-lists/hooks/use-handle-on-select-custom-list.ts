import React from 'react';

import type { RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useOnSelectEntryLocation, useOnSelectExitLocation } from '../../../hooks';
import { useCustomListsContext } from '../CustomListsContext';

export function useHandleOnSelectCustomList() {
  const [onSelectExitRelay] = useOnSelectExitLocation();
  const [onSelectEntryRelay] = useOnSelectEntryLocation();
  const { locationSelection } = useCustomListsContext();

  return React.useCallback(
    async (value: RelayLocation) => {
      const location = { ...value };
      if ('country' in location) {
        // Only the geographical part should be sent to the daemon when setting a location.
        delete location.customList;
      }
      if (locationSelection === 'entry') {
        await onSelectEntryRelay(location);
      } else {
        await onSelectExitRelay(location);
      }
    },
    [locationSelection, onSelectEntryRelay, onSelectExitRelay],
  );
}
