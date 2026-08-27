import React from 'react';
import styled, { css } from 'styled-components';

import { Gallery } from '../../../../../gallery';
import { useCarouselContext } from '../../../../CarouselContext';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const StyledSlide = styled.div<{ $active: boolean }>`
  ${({ $active }) => css`
    width: 100%;
    display: inline-block;
    white-space: normal;
    vertical-align: top;
    scroll-snap-align: start;

    opacity: ${$active ? 1 : 0.25};

    transition: opacity 0.25s ease-in-out;
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
      aria-hidden={!isActiveSlide}
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
