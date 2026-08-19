import React from 'react';
import styled from 'styled-components';

import { Flex } from '../flex';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import { CarouselControls, CarouselSlides } from './components';
import { useEffectScrollToSlide, useFocusCarousel, useHandleKeyboardNavigation } from './hooks';

export type CarouselProps = React.ComponentPropsWithRef<'section'> & {
  slideIndex?: number;
  onSlideIndexChange?: (slideIndex: number) => void;
};

export const StyledCarousel = styled.section`
  width: 100%;
`;

export const StyledFlex = styled(Flex)`
  width: 100%;
`;

function CarouselImpl({ children, ...props }: CarouselProps) {
  const handleKeyboardNavigation = useHandleKeyboardNavigation();
  const { carouselRef } = useCarouselContext();

  useEffectScrollToSlide();
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

function Carousel({ slideIndex, onSlideIndexChange, children, ...props }: CarouselProps) {
  return (
    <CarouselProvider slideIndex={slideIndex} onSlideIndexChange={onSlideIndexChange}>
      <CarouselImpl {...props}>{children}</CarouselImpl>
    </CarouselProvider>
  );
}

const CarouselNamespace = Object.assign(Carousel, {
  Slides: CarouselSlides,
  Controls: CarouselControls,
});

export { CarouselNamespace as Carousel };
