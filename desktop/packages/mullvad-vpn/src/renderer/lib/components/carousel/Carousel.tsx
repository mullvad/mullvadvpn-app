import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Flex } from '../flex';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import {
  CarouselControls,
  CarouselSlides,
  StyledCarouselControls,
  StyledCarouselSlides,
} from './components';
import { useFocusCarousel, useHandleKeyboardNavigation } from './hooks';

export type CarouselProps = React.ComponentPropsWithRef<'section'>;

export const StyledCarousel = styled.section`
  width: 100%;
`;

export const StyledFlex = styled(Flex)`
  width: 100%;

  > ${StyledCarouselSlides} + ${StyledCarouselControls} {
    margin-top: ${spacings.medium};
  }
`;

function CarouselImpl({ children, ...props }: CarouselProps) {
  const handleKeyboardNavigation = useHandleKeyboardNavigation();
  const { carouselRef } = useCarouselContext();

  useFocusCarousel();

  return (
    <StyledCarousel
      ref={carouselRef}
      onKeyDown={handleKeyboardNavigation}
      aria-roledescription="carousel"
      tabIndex={-1}
      {...props}>
      <StyledFlex flexDirection="column">{children}</StyledFlex>
    </StyledCarousel>
  );
}

function Carousel({ children, ...props }: CarouselProps) {
  return (
    <CarouselProvider>
      <CarouselImpl {...props}>{children}</CarouselImpl>
    </CarouselProvider>
  );
}

const CarouselNamespace = Object.assign(Carousel, {
  Slides: CarouselSlides,
  Controls: CarouselControls,
});

export { CarouselNamespace as Carousel };
