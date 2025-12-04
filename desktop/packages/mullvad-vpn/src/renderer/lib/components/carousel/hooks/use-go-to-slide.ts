import React from 'react';

import { useCarouselContext } from '../CarouselContext';

// Scroll to a specific slide.
export function useGoToSlide() {
  const { slidesRef } = useCarouselContext();
  return React.useCallback(
    (slide: number) => {
      if (slidesRef.current) {
        const width = slidesRef.current.offsetWidth;
        slidesRef.current.scrollTo({ left: width * slide });
      }
    },
    [slidesRef],
  );
}
