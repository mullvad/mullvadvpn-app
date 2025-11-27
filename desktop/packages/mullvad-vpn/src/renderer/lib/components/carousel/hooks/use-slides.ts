import React from 'react';

import { useCarouselContext } from '../CarouselContext';
import { useGetSlideIndex } from './use-get-slide-index';
import { useGoToSlide } from './use-go-to-slide';

export function useSlides() {
  const { slides, slideIndex } = useCarouselContext();
  const goToSlide = useGoToSlide();
  const getSlideIndex = useGetSlideIndex();

  const isFirstSlide = slideIndex === 0;
  const isLastSlide = slideIndex === slides.length - 1;

  const goToNextSlide = React.useCallback(() => {
    if (!isLastSlide) {
      goToSlide(getSlideIndex() + 1);
    }
  }, [isLastSlide, goToSlide, getSlideIndex]);

  const goToPreviousSlide = React.useCallback(() => {
    if (!isFirstSlide) {
      goToSlide(getSlideIndex() - 1);
    }
  }, [isFirstSlide, goToSlide, getSlideIndex]);

  return {
    goToSlide,
    goToNextSlide,
    goToPreviousSlide,
    isFirstSlide,
    isLastSlide,
  };
}
