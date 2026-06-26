import React from 'react';

import { LocationType } from '../../../../../../features/locations/types';
import { useScrollPositionContext } from '../../../ScrollPositionContext';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectedItemChange() {
  const { saveScrollPosition } = useScrollPositionContext();
  const { setLocationType } = useSelectLocationViewContext();

  return React.useCallback(
    (id: string) => {
      saveScrollPosition();
      if (id === 'entry') {
        setLocationType(LocationType.entry);
      } else {
        setLocationType(LocationType.exit);
      }
    },
    [saveScrollPosition, setLocationType],
  );
}
