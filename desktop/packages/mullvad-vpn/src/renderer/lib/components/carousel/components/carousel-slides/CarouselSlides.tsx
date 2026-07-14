import React from 'react';
import styled, { css } from 'styled-components';

import type { TransientProps } from '../../../../types';
import { useCarouselContext } from '../../CarouselContext';
import { useGetSlideIndex } from '../../hooks';
import { CarouselSlide } from './components';

export type CarouselSlidesProps = React.ComponentPropsWithRef<'div'>;

type StyledSlidesProps = TransientProps<{
  disableScroll: boolean;
}>;

const StyledSlides = styled.div<StyledSlidesProps>`
  ${({ $disableScroll }) => {
    return css`
      white-space: nowrap;
      overflow: ${$disableScroll ? 'hidden' : 'scroll hidden'};
      scroll-snap-type: x mandatory;
      scroll-behavior: smooth;

      &&::-webkit-scrollbar {
        display: none;
      }
    `;
  }};
`;

function CarouselSlides({ children, ...props }: CarouselSlidesProps) {
  const { disableScroll, slidesRef, setSlideIndex } = useCarouselContext();
  const getSlideIndex = useGetSlideIndex();

  // Update slide number after scrolling.
  const handleScroll = React.useCallback(() => {
    return setSlideIndex(getSlideIndex());
  }, [getSlideIndex, setSlideIndex]);

  return (
    <StyledSlides
      ref={slidesRef}
      onScrollEnd={handleScroll}
      $disableScroll={disableScroll}
      aria-live="polite"
      aria-atomic="true"
      tabIndex={-1}
      {...props}>
      {children}
    </StyledSlides>
  );
}

const CarouselSlidesNamespace = Object.assign(CarouselSlides, {
  Slide: CarouselSlide,
});

export { CarouselSlidesNamespace as CarouselSlides };
