import React from 'react';

import { useProviders } from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useActiveProviders() {
  const { entryProviders, exitProviders, setEntryProviders, setExitProviders } = useProviders();
  const { locationType } = useSelectLocationViewContext();
  const activeProviders = locationType === LocationType.entry ? entryProviders : exitProviders;

  const setActiveProviders = React.useCallback(
    async (providers: string[]) => {
      if (locationType === LocationType.entry) {
        await setEntryProviders(providers);
      }

      if (locationType === LocationType.exit) {
        await setExitProviders(providers);
      }
    },
    [locationType, setEntryProviders, setExitProviders],
  );

  return { activeProviders, setActiveProviders };
}
