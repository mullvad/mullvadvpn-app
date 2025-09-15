import React from 'react';

export const useFocusReference = <T extends HTMLElement = HTMLDivElement>(
  ref?: React.RefObject<T | null>,
  focus?: boolean,
) => {
  React.useEffect(() => {
    if (focus) {
      ref?.current?.focus({ preventScroll: true });
    }
  }, [ref, focus]);
};
