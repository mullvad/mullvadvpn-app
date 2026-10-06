import React from 'react';

import { LocationType } from '../../../../../../features/locations/types';
import type { LocationSelectorItemType } from '../../../../../../lib/components/location-selector/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectedItemChange() {
  const { setLocationType } = useSelectLocationViewContext();

  return React.useCallback(
    (id: LocationSelectorItemType) => {
      if (id === 'entry') {
        setLocationType(LocationType.entry);
      } else if (id === 'automaticEntry') {
        setLocationType(LocationType.entryAutomatic);
      } else {
        setLocationType(LocationType.exit);
      }
    },
    [setLocationType],
  );
}
