import React from 'react';

import { useCarouselContext } from '../CarouselContext';

export function useGoToSlide() {
  const { onSlideIndexChange } = useCarouselContext();

  return React.useCallback(
    (slideIndex: number) => {
      onSlideIndexChange?.(slideIndex);
    },
    [onSlideIndexChange],
  );
}
