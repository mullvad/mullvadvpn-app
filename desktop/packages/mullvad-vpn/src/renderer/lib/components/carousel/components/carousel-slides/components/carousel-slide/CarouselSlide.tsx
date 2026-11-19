import React from 'react';
import styled, { css } from 'styled-components';

import { durations } from '../../../../../../foundations';
import { Gallery } from '../../../../../gallery';
import { useCarouselContext } from '../../../../CarouselContext';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const StyledSlide = styled.div<{ $active: boolean }>`
  ${({ $active }) => css`
    flex: 0 0 100%;

    white-space: normal;
    vertical-align: top;
    scroll-snap-align: start;

    opacity: ${$active ? 1 : 0.25};

    transition: opacity ${durations.normal} ease-in-out;
  `}
`;

function CarouselSlide({ children, ...props }: CarouselSlideProps) {
  const id = React.useId();
  const { slides, slideIndex } = useCarouselContext();
  const isActiveSlide = slides[slideIndex]?.id === id;

  return (
    <StyledSlide
      id={id}
      tabIndex={-1}
      inert={!isActiveSlide}
      data-carousel-slide
      role="group"
      aria-roledescription="slide"
      $active={isActiveSlide}
      {...props}>
      <Gallery>{children}</Gallery>
    </StyledSlide>
  );
}

const CarouselSlideNamespace = Object.assign(CarouselSlide, {
  Text: Gallery.Text,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
});

export { CarouselSlideNamespace as CarouselSlide };
