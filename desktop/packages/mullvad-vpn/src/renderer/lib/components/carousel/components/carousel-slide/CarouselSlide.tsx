import React from 'react';
import styled from 'styled-components';

import { Gallery } from '../../../gallery';
import { useCarouselContext } from '../../CarouselContext';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const StyledSlide = styled.div`
  display: inline-block;
  width: 100%;
  white-space: normal;
  vertical-align: top;
  scroll-snap-align: start;
`;

export function CarouselSlide({ children, ...props }: CarouselSlideProps) {
  const id = React.useId();
  const { slides, slideIndex } = useCarouselContext();
  const isActiveSlide = slides[slideIndex]?.id === id;
  return (
    <StyledSlide id={id} aria-hidden={!isActiveSlide} data-carousel-slide {...props}>
      <Gallery>{children}</Gallery>
    </StyledSlide>
  );
}
