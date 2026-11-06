import React from 'react';

import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';
import { useResetLocationSelector } from './use-reset-location-selector';

export function useSetMultihopWhenNeeded() {
  const { locationType, setLocationType } = useSelectLocationViewContext();
  const resetLocationSelector = useResetLocationSelector();
  const { setMultihop } = useMultihop();

  const setMultihopWhenNeeded = React.useCallback(async () => {
    await setMultihop({ multihop: 'when-needed' });
    if (locationType === 'entry' || locationType === 'automaticEntry') {
      resetLocationSelector();
      setLocationType('exit');
    }
  }, [locationType, resetLocationSelector, setLocationType, setMultihop]);

  return setMultihopWhenNeeded;
}
