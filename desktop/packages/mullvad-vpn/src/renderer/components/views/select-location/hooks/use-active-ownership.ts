import React from 'react';

import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { useOwnership } from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useActiveOwnership() {
  const { entryOwnership, exitOwnership, setEntryOwnership, setExitOwnership } = useOwnership();
  const { locationType } = useSelectLocationViewContext();
  const activeOwnership = locationType === LocationType.entry ? entryOwnership : exitOwnership;

  const setActiveOwnership = React.useCallback(
    async (ownership: Ownership) => {
      if (locationType === LocationType.entry) {
        await setEntryOwnership(ownership);
      }

      if (locationType === LocationType.exit) {
        await setExitOwnership(ownership);
      }
    },
    [locationType, setEntryOwnership, setExitOwnership],
  );

  return { activeOwnership, setActiveOwnership };
}
