import React from 'react';

import { useCarouselContext } from '../CarouselContext';

// Scroll to a specific slide.
export function useEffectScrollToSlide() {
  const { slidesRef, slideIndex } = useCarouselContext();
  return React.useEffect(() => {
    if (slidesRef.current) {
      const width = slidesRef.current.offsetWidth;
      slidesRef.current.scrollTo({ left: width * slideIndex });
    }
  }, [slidesRef, slideIndex]);
}
