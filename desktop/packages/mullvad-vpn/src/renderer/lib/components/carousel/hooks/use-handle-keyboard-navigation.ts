import React from 'react';

import { useSlides } from './use-slides';

export const useHandleKeyboardNavigation = () => {
  const { next, prev } = useSlides();
  return React.useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        prev();
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        next();
      }
    },
    [next, prev],
  );
};
