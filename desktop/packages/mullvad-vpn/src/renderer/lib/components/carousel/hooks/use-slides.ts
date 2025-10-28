import React from 'react';

import { useCarouselContext } from '../CarouselContext';
import { useGetSlideIndex } from './use-get-slide-index';
import { useGoToSlide } from './use-go-to-slide';

export function useSlides() {
  const { content, slideIndex } = useCarouselContext();
  const goToSlide = useGoToSlide();
  const getSlideIndex = useGetSlideIndex();

  // These values are only intended to be used for display purposes. Using them when calculating
  // next or prev slide would increase the risk of race conditions.
  const hasNext = slideIndex < content.length - 1;
  const hasPrev = slideIndex > 0;

  const next = React.useCallback(() => goToSlide(getSlideIndex() + 1), [goToSlide, getSlideIndex]);
  const prev = React.useCallback(() => goToSlide(getSlideIndex() - 1), [goToSlide, getSlideIndex]);

  return {
    goToSlide,
    next,
    prev,
    hasNext,
    hasPrev,
  };
}
