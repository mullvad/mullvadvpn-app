import React from 'react';

export const useFocusReferenceAfterPaint = <T extends HTMLElement>(
  ref?: React.RefObject<T | null>,
  focus?: boolean,
) => {
  React.useEffect(() => {
    console.log('inside focus after paint', ref?.current, document.activeElement?.tagName);
    if (focus && ref?.current) {
      console.log('focusing?', ref);
      // ref?.current?.focus({ preventScroll: true });
    }
  }, [ref, focus]);
};
