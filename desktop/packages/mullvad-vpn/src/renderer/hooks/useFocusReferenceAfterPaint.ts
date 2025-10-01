import React from 'react';

export const useFocusReferenceAfterPaint = <T extends HTMLElement>(
  ref?: React.RefObject<T | null>,
  focus?: boolean,
) => {
  React.useEffect(() => {
    if (focus) {
      ref?.current?.focus({ preventScroll: true });
    }
  }, [ref, focus]);
};
