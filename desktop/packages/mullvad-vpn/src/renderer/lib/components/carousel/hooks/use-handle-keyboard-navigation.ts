import React from 'react';

import { useSlides } from './use-slides';

export const useHandleKeyboardNavigation = () => {
  const { goToNextSlide, goToPreviousSlide } = useSlides();
  return React.useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        goToPreviousSlide();
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        goToNextSlide();
      }
    },
    [goToNextSlide, goToPreviousSlide],
  );
};
