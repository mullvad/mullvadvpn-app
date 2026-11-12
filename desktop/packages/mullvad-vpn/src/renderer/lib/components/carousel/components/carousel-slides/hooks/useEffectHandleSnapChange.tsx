import React from 'react';

import { useCarouselContext } from '../../../CarouselContext';
import { useSlides } from '../../../hooks';

export function useEffectHandleSnapChange() {
  const { slidesRef } = useCarouselContext();
  const { goToSlide } = useSlides();

  React.useEffect(() => {
    const slides = slidesRef.current;
    if (!slides) {
      return;
    }

    const handleSnapChange = () => {
      const width = slides.offsetWidth;
      const slideIndex = Math.round(slides.scrollLeft / width);
      goToSlide(slideIndex);
    };

    slides.addEventListener('scrollsnapchange', handleSnapChange);

    return () => {
      return slides.removeEventListener('scrollsnapchange', handleSnapChange);
    };
  }, [goToSlide, slidesRef]);
}
