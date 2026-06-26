import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleReset() {
  const {
    textField: { reset },
  } = useSelectLocationSelectorItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleReset = React.useCallback(() => {
    setIsolatedItem(undefined);
    React.startTransition(() => {
      reset();
      setSearchTerm('');
    });
  }, [reset, setIsolatedItem, setSearchTerm]);

  return handleReset;
}
