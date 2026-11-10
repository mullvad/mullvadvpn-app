import React from 'react';

import { useProviders } from '../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useActiveProviders() {
  const { entryProviders, exitProviders, setEntryProviders, setExitProviders } = useProviders();
  const { locationType } = useSelectLocationViewContext();
  const activeProviders = locationType === 'entry' ? entryProviders : exitProviders;

  const setActiveProviders = React.useCallback(
    async (providers: string[]) => {
      if (locationType === 'entry') {
        await setEntryProviders(providers);
      }

      if (locationType === 'exit') {
        await setExitProviders(providers);
      }
    },
    [locationType, setEntryProviders, setExitProviders],
  );

  return { activeProviders, setActiveProviders };
}
