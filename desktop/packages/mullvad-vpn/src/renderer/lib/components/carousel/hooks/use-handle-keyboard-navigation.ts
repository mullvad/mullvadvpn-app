import React from 'react';

export const useHandleOptionsKeyboardNavigation = ({
  next,
  previous,
}: {
  next: () => void;
  previous: () => void;
}) => {
  return React.useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        previous();
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        next();
      }
    },
    [next, previous],
  );
};
