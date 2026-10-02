import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useTextFieldItemContext } from '../TextFieldItemContext';
import { focusFirstFocusableHeading } from '../utils';
import { useHandleReset } from './use-handle-reset';

export function useHandleInputKeyDown() {
  const { id } = useTextFieldItemContext();
  const { setIsolatedItem, searchTerm } = useSelectLocationViewContext();
  const handleReset = useHandleReset();

  return React.useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        if (searchTerm) {
          focusFirstFocusableHeading();
          setIsolatedItem(id);
        }
      }

      if (event.key === 'Escape') {
        event.preventDefault();
        handleReset();
      }
    },
    [searchTerm, setIsolatedItem, id, handleReset],
  );
}
