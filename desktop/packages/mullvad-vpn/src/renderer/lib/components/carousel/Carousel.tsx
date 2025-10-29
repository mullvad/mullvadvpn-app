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

export type CarouselProps = React.ComponentPropsWithRef<'div'>;

function CarouselImpl({ children }: Pick<CarouselProps, 'children'>) {
  const handleKeyboardNavigation = useHandleKeyboardNavigation();

  return (
    <StyledCarousel $flexDirection="column" $gap="medium" onKeyDown={handleKeyboardNavigation}>
      {children}
    </StyledCarousel>
  );
}

function Carousel({ children }: CarouselProps) {
  return (
    <CarouselProvider>
      <CarouselImpl>{children}</CarouselImpl>
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
