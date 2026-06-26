import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../../../../lib/components/location-selector';
import { useSelectLocationSelectorContext } from '../../../SelectLocationSelectorContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useEffectSetIsolatedItem(id: LocationSelectorSelectedItem) {
  const { setIsolatedItem } = useSelectLocationSelectorContext();
  const { searching } = useSelectLocationSelectorItemContext();

  React.useEffect(() => {
    if (searching) {
      setIsolatedItem(id);
    } else {
      setIsolatedItem(undefined);
    }
  }, [searching, id, setIsolatedItem]);
}
