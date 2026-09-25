import React from 'react';
import styled from 'styled-components';

import { Flex } from '../flex';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import { CarouselControls, CarouselSlides } from './components';
import { useEffectScrollToSlide, useFocusCarousel, useHandleKeyboardNavigation } from './hooks';

export const StyledCarousel = styled.section``;

export type CarouselProps = React.ComponentPropsWithRef<'section'> & {
  slideIndex?: number;
  onSlideIndexChange?: (slideIndex: number) => void;
  onSlideSettled?: (slideIndex: number) => void;
  disableScroll?: boolean;
};

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
      <Flex flexDirection="column" gap="medium">
        {children}
      </Flex>
    </StyledCarousel>
  );
}

function Carousel({
  slideIndex,
  onSlideIndexChange,
  onSlideSettled,
  disableScroll,
  children,
  ...props
}: CarouselProps) {
  return (
    <CarouselProvider
      slideIndex={slideIndex}
      onSlideIndexChange={onSlideIndexChange}
      onSlideSettled={onSlideSettled}
      disableScroll={disableScroll}>
      <CarouselImpl {...props}>{children}</CarouselImpl>
    </CarouselProvider>
  );
}

const CarouselNamespace = Object.assign(Carousel, {
  Slides: CarouselSlides,
  Controls: CarouselControls,
});

export { CarouselNamespace as Carousel };
