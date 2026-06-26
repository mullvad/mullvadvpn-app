import React from 'react';

import { useSelectLocationSelectorContext } from '../../../SelectLocationSelectorContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useEffectSetIsolatedItem(id: string) {
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
