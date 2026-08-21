import React from 'react';

import { LocationType } from '../../../../../../features/locations/types';
import { type LocationSelectorSelectedItem } from '../../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectedItemChange() {
  const { setLocationType } = useSelectLocationViewContext();

  return React.useCallback(
    (id: LocationSelectorSelectedItem) => {
      if (id === 'entry') {
        setLocationType(LocationType.entry);
      } else {
        setLocationType(LocationType.exit);
      }
    },
    [setLocationType],
  );
}
