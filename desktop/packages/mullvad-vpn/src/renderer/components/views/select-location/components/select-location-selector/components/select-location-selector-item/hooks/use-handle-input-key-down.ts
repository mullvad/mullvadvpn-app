import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleInputKeyDown() {
  const {
    triggerRef,
    focused,
    setFocused,
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
        if (focused) {
          reset();
          setFocused(false);
          triggerRef.current?.focus();
        }
      }
    },
    [focusFirstFocusableHeading, focused, reset, setFocused, triggerRef],
  );
}
