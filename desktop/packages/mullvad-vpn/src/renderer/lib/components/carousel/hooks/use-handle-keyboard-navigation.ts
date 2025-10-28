import React from 'react';

import { usePages } from './use-pages';

export const useHandleKeyboardNavigation = () => {
  const { next, prev } = usePages();
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
