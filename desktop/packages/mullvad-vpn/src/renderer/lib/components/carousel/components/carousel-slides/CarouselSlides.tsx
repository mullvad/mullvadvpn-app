import React from 'react';
import styled from 'styled-components';

import { useCarouselContext } from '../../CarouselContext';
import { useGetSlideIndex } from '../../hooks';

export type CarouselSlidesProps = React.ComponentPropsWithRef<'div'>;

const StyledSlides = styled.div`
  white-space: nowrap;
  overflow: scroll hidden;
  scroll-snap-type: x mandatory;
  scroll-behavior: smooth;

  &&::-webkit-scrollbar {
    display: none;
  }
`;

export function CarouselSlides({ children, ...props }: CarouselSlidesProps) {
  const { slidesRef, setSlideIndex } = useCarouselContext();
  const getSlideIndex = useGetSlideIndex();

  // Update slide number after scrolling.
  const handleScroll = React.useCallback(() => {
    return setSlideIndex(getSlideIndex());
  }, [getSlideIndex, setSlideIndex]);

  return (
    <StyledSlides
      ref={slidesRef}
      onScrollEnd={handleScroll}
      aria-live="polite"
      aria-atomic="true"
      tabIndex={-1}
      {...props}>
      {children}
    </StyledSlides>
  );
}
