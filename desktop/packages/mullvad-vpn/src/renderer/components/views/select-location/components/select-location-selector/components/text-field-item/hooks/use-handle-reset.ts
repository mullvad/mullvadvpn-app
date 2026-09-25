import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useTextFieldItemContext } from '../TextFieldItemContext';

export function useHandleReset() {
  const {
    textField: { reset },
  } = useTextFieldItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleReset = React.useCallback(() => {
    setIsolatedItem(undefined);
    reset();
    setSearchTerm('');
  }, [reset, setIsolatedItem, setSearchTerm]);

  return handleReset;
}
