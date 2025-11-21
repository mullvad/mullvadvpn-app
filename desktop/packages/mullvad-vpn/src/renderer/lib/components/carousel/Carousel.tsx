import React from 'react';
import styled from 'styled-components';

import { Flex } from '../flex';
import { Gallery } from '../gallery';
import { CarouselProvider } from './CarouselContext';
import {
  CarouselControlGroup,
  CarouselControls,
  CarouselIndicator,
  CarouselIndicators,
  CarouselNextButton,
  CarouselPrevButton,
  CarouselSlide,
  CarouselSlides,
} from './components';
import { useHandleKeyboardNavigation } from './hooks';

const StyledCarousel = styled(Flex)``;

export type CarouselProps = React.ComponentPropsWithRef<'section'>;

function CarouselImpl({ children, ...props }: CarouselProps) {
  const handleKeyboardNavigation = useHandleKeyboardNavigation();

  return (
    <StyledCarousel
      as={'section'}
      $flexDirection="column"
      $gap="medium"
      onKeyDown={handleKeyboardNavigation}
      aria-roledescription="carousel"
      {...props}>
      {children}
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
  Indicator: CarouselIndicator,
  NextButton: CarouselNextButton,
  PrevButton: CarouselPrevButton,
  Indicators: CarouselIndicators,
});

export { CarouselNamespace as Carousel };
