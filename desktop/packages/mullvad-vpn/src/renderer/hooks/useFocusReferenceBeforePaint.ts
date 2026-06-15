import React from 'react';

export const useFocusReferenceBeforePaint = <T extends HTMLElement>(
  ref?: React.RefObject<T | null>,
  focus?: boolean,
) => {
  React.useLayoutEffect(() => {
    if (focus) {
      console.log('focus before paint?');
      ref?.current?.focus({ preventScroll: true });
    }
  }, [ref, focus]);
};
