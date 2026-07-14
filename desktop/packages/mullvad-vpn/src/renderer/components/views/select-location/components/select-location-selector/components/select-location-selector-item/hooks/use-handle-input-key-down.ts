import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';
import { useFocusFirstFocusableHeading } from './use-focus-first-focusable-heading';
import { useHandleReset } from './use-handle-reset';

export function useHandleInputKeyDown() {
  const {
    setSearching,
    textField: { value },
  } = useSelectLocationSelectorItemContext();
  const { setSearchTerm } = useSelectLocationViewContext();
  const handleReset = useHandleReset();
  const focusFirstFocusableHeading = useFocusFirstFocusableHeading();

  return React.useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        setSearchTerm(value);
        setSearching(false);
        focusFirstFocusableHeading();
      }

      if (event.key === 'Escape') {
        event.preventDefault();
        handleReset();
      }
    },
    [setSearchTerm, value, setSearching, focusFirstFocusableHeading, handleReset],
  );
}
