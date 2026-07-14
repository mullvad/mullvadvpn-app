import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleReset() {
  const {
    setSearching,
    textField: { reset },
  } = useSelectLocationSelectorItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleReset = React.useCallback(() => {
    setIsolatedItem(undefined);
    setSearching(false);
    React.startTransition(() => {
      reset();
      setSearchTerm('');
    });
  }, [reset, setIsolatedItem, setSearchTerm, setSearching]);

  return handleReset;
}
