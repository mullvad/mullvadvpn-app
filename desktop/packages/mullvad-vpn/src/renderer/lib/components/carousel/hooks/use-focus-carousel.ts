import React from 'react';

import { useCarouselContext } from '../CarouselContext';
import { useSlides } from './use-slides';

// Move focus to carousel when reaching the first or last slide, since the buttons get disabled.
export function useFocusCarousel() {
  const { isFirstSlide, isLastSlide } = useSlides();
  const { carouselRef, nextButtonRef, prevButtonRef, firstIndicatorRef, lastIndicatorRef } =
    useCarouselContext();
  React.useEffect(() => {
    const focusedElement = document.activeElement;
    if (
      isFirstSlide &&
      (focusedElement === prevButtonRef?.current || focusedElement === firstIndicatorRef?.current)
    ) {
      carouselRef?.current?.focus();
    } else if (
      isLastSlide &&
      (focusedElement === nextButtonRef?.current || focusedElement === lastIndicatorRef?.current)
    ) {
      carouselRef?.current?.focus();
    }
  }, [
    carouselRef,
    isFirstSlide,
    isLastSlide,
    nextButtonRef,
    prevButtonRef,
    firstIndicatorRef,
    lastIndicatorRef,
  ]);
}
