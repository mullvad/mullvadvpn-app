import React from 'react';

export function useFocusFirstFocusableHeading() {
  const focusFirstFocusableHeading = React.useCallback(() => {
    const firstFocusableHeading = document.querySelector<HTMLElement>('[data-focusable-heading]');
    firstFocusableHeading?.focus();
  }, []);

  return focusFirstFocusableHeading;
}
