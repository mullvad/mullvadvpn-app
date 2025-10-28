import { useCallback } from 'react';
import styled from 'styled-components';

import { NonEmptyArray } from '../../../../shared/utils';
import { Flex } from '../flex';
import { Gallery } from '../gallery';
import { CarouselProvider, useCarouselContext } from './CarouselContext';
import { CarouselControls } from './components';
import { useGetPageNumber, useHandleKeyboardNavigation } from './hooks';

const PAGE_GAP = 16;

const StyledCarousel = styled(Flex)``;

const StyledPageSlider = styled.div({
  whiteSpace: 'nowrap',
  overflow: 'scroll hidden',
  scrollSnapType: 'x mandatory',
  scrollBehavior: 'smooth',

  '&&::-webkit-scrollbar': {
    display: 'none',
  },
});

const StyledPage = styled.div({
  display: 'inline-block',
  width: '100%',
  whiteSpace: 'normal',
  verticalAlign: 'top',
  scrollSnapAlign: 'start',

  '&&:not(:last-child)': {
    marginRight: `${PAGE_GAP}px`,
  },
});

export interface PageSliderProps {
  content: NonEmptyArray<React.ReactNode>;
}

function CarouselImpl() {
  const { pageContainerRef, setPageNumber, content } = useCarouselContext();
  const handleKeyboardNavigation = useHandleKeyboardNavigation();

  const getPageNumber = useGetPageNumber();

  // Trigger a rerender when the page number has changed. This needs to be done to update the
  // states of the arrows and page indicators.
  const handleScroll = useCallback(() => {
    return setPageNumber(getPageNumber());
  }, [getPageNumber, setPageNumber]);

  return (
    <StyledCarousel $flexDirection="column" $gap="medium" onKeyDown={handleKeyboardNavigation}>
      <StyledPageSlider ref={pageContainerRef} onScrollEnd={handleScroll}>
        {content.map((page, i) => (
          <StyledPage key={`page-${i}`}>{page}</StyledPage>
        ))}
      </StyledPageSlider>
      <CarouselControls />
    </StyledCarousel>
  );
}

function Carousel({ content }: PageSliderProps) {
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
  Page: Gallery,
});

export { CarouselNamespace as Carousel };
