import React from 'react';

import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { useOwnership } from '../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useActiveOwnership() {
  const { entryOwnership, exitOwnership, setEntryOwnership, setExitOwnership } = useOwnership();
  const { locationType } = useSelectLocationViewContext();
  const activeOwnership = locationType === 'entry' ? entryOwnership : exitOwnership;

  const setActiveOwnership = React.useCallback(
    async (ownership: Ownership) => {
      if (locationType === 'entry') {
        await setEntryOwnership(ownership);
      }

      if (locationType === 'exit') {
        await setExitOwnership(ownership);
      }
    },
    [locationType, setEntryOwnership, setExitOwnership],
  );

  return { activeOwnership, setActiveOwnership };
}
