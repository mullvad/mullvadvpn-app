import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleInputKeyDown() {
  const {
    triggerRef,
    searching,
    textField: { reset },
  } = useSelectLocationSelectorItemContext();

  const focusFirstFocusableHeading = React.useCallback(() => {
    const firstFocusableHeading = document.querySelector<HTMLElement>('[data-focusable-heading]');
    firstFocusableHeading?.focus();
  }, []);

  return React.useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        focusFirstFocusableHeading();
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        if (searching) {
          reset();
          triggerRef.current?.focus();
        }
      }
    },
    [focusFirstFocusableHeading, searching, reset, triggerRef],
  );
}
