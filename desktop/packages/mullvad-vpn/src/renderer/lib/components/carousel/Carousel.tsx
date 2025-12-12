import React from 'react';
import styled from 'styled-components';

import { Flex } from '../flex';
import { Gallery } from '../gallery';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import {
  CarouselControlGroup,
  CarouselControls,
  CarouselIndicators,
  CarouselNextButton,
  CarouselPrevButton,
  CarouselSlide,
  CarouselSlides,
} from './components';
import { useFocusCarousel, useHandleKeyboardNavigation } from './hooks';

export const StyledCarousel = styled.section``;

export type CarouselProps = React.ComponentPropsWithRef<'section'>;

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
      <Flex flexDirection="column" gap="medium">
        {children}
      </Flex>
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
  Text: Gallery.Text,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
  Slide: CarouselSlide,
  Slides: CarouselSlides,
  Controls: CarouselControls,
  ControlGroup: CarouselControlGroup,
  NextButton: CarouselNextButton,
  PrevButton: CarouselPrevButton,
  Indicators: CarouselIndicators,
});

export { CarouselNamespace as Carousel };
