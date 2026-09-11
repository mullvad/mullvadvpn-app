import React from 'react';
import styled, { css } from 'styled-components';

import type { TransientProps } from '../../../../types';
import { useMounted } from '../../../../utility-hooks';
import { useCarouselContext } from '../../CarouselContext';
import { CarouselSlide } from './components';

export type CarouselSlidesProps = React.ComponentPropsWithRef<'div'>;

type StyledSlidesProps = TransientProps<{
  disableScroll: boolean;
  mounted: boolean;
}>;

const StyledSlides = styled.div<StyledSlidesProps>`
  ${({ $disableScroll, $mounted }) => {
    return css`
      white-space: nowrap;
      overflow: ${$disableScroll ? 'hidden' : 'scroll hidden'};
      scroll-snap-type: x mandatory;
      scroll-behavior: ${$mounted ? 'smooth' : 'auto'};

      &&::-webkit-scrollbar {
        display: none;
      }
    `;
  }};
`;

function CarouselSlides({ children, ...props }: CarouselSlidesProps) {
  const { disableScroll, slidesRef, onSlideSettled, slideIndex } = useCarouselContext();

  const isMounted = useMounted();
  const mounted = isMounted();

  const handleScrollEnd = React.useCallback(() => {
    if (onSlideSettled) {
      onSlideSettled(slideIndex);
    }
  }, [onSlideSettled, slideIndex]);

  return (
    <StyledSlides
      ref={slidesRef}
      $disableScroll={disableScroll}
      $mounted={mounted}
      aria-live="polite"
      aria-atomic="true"
      tabIndex={-1}
      onScrollEnd={handleScrollEnd}
      {...props}>
      {children}
    </StyledSlides>
  );
}

const CarouselSlidesNamespace = Object.assign(CarouselSlides, {
  Slide: CarouselSlide,
});

export { CarouselSlidesNamespace as CarouselSlides };
