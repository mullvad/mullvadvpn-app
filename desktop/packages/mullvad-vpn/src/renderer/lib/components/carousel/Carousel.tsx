import { useCallback } from 'react';
import styled from 'styled-components';

import { NonEmptyArray } from '../../../../shared/utils';
import { Flex } from '../flex';
import { Gallery } from '../gallery';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import { CarouselControls } from './components';
import { useGetSlideIndex, useHandleKeyboardNavigation } from './hooks';

const SLIDE_GAP = 16;

const StyledCarousel = styled(Flex)``;

const StyledSlides = styled.div({
  whiteSpace: 'nowrap',
  overflow: 'scroll hidden',
  scrollSnapType: 'x mandatory',
  scrollBehavior: 'smooth',

  '&&::-webkit-scrollbar': {
    display: 'none',
  },
});

const StyledSlide = styled.div({
  display: 'inline-block',
  width: '100%',
  whiteSpace: 'normal',
  verticalAlign: 'top',
  scrollSnapAlign: 'start',

  '&&:not(:last-child)': {
    marginRight: `${SLIDE_GAP}px`,
  },
});

export interface CarouselProps {
  content: NonEmptyArray<React.ReactNode>;
}

function CarouselImpl() {
  const { slidesRef, setSlideIndex, content } = useCarouselContext();
  const handleKeyboardNavigation = useHandleKeyboardNavigation();

  const getSlideIndex = useGetSlideIndex();

  // Update slide number after scrolling.
  const handleScroll = useCallback(() => {
    return setSlideIndex(getSlideIndex());
  }, [getSlideIndex, setSlideIndex]);

  return (
    <StyledCarousel $flexDirection="column" $gap="medium" onKeyDown={handleKeyboardNavigation}>
      <StyledSlides ref={slidesRef} onScrollEnd={handleScroll}>
        {content.map((slide, i) => (
          <StyledSlide key={`slide-${i}`}>{slide}</StyledSlide>
        ))}
      </StyledSlides>
      <CarouselControls />
    </StyledCarousel>
  );
}

function Carousel({ content }: CarouselProps) {
  return (
    <CarouselProvider content={content}>
      <CarouselImpl />
    </CarouselProvider>
  );
}

const CarouselNamespace = Object.assign(Carousel, {
  Text: Gallery.Text,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
  Slide: Gallery,
});

export { CarouselNamespace as Carousel };
