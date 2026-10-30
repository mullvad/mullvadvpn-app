import React from 'react';

import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useResetLocationSelector() {
  const { setIsolatedItem, setSearchTerm } = useSelectLocationViewContext();
  const resetLocationSelector = React.useCallback(() => {
    setIsolatedItem(undefined);
    setSearchTerm('');
  }, [setIsolatedItem, setSearchTerm]);

  return resetLocationSelector;
}
