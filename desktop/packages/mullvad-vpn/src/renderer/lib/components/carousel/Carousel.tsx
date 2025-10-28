import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import { NonEmptyArray } from '../../../../shared/utils';
import { useStyledRef } from '../../utility-hooks';
import { Flex } from '../flex';
import { CarouselControls } from './components';

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

interface PageSliderProps {
  content: NonEmptyArray<React.ReactNode>;
}

export function Carousel(props: PageSliderProps) {
  // A state is needed to trigger a rerender. This is needed to update the "disabled" and "$current"
  // props of the arrows and page indicators.
  const [, setPageNumberState] = useState(0);
  const pageContainerRef = useStyledRef<HTMLDivElement>();

  // Calculate the page number based on the scroll position.
  const getPageNumber = useCallback(() => {
    if (pageContainerRef.current) {
      const scrollLeft = pageContainerRef.current.scrollLeft;
      const pageWidth = pageContainerRef.current.offsetWidth + PAGE_GAP;
      // Clamp it between 0 and props.content.length-1 to make sure it will correspond to a page.
      return Math.max(0, Math.min(Math.round(scrollLeft / pageWidth), props.content.length - 1));
    } else {
      return 0;
    }
  }, [pageContainerRef, props.content.length]);

  // These values are only intended to be used for display purposes. Using them when calculating
  // next or prev page would increase the risk of race conditions.
  const pageNumber = getPageNumber();
  const hasNext = pageNumber < props.content.length - 1;
  const hasPrev = pageNumber > 0;

  // Scroll to a specific page.
  const goToPage = useCallback(
    (page: number) => {
      if (pageContainerRef.current) {
        const width = pageContainerRef.current.offsetWidth;
        pageContainerRef.current.scrollTo({ left: width * page });
      }
    },
    [pageContainerRef],
  );

  const next = useCallback(() => goToPage(getPageNumber() + 1), [goToPage, getPageNumber]);
  const prev = useCallback(() => goToPage(getPageNumber() - 1), [goToPage, getPageNumber]);

  // Callback that navigates when left and right arrows are pressed.
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'ArrowLeft') {
        prev();
      } else if (event.key === 'ArrowRight') {
        next();
      }
    },
    [next, prev],
  );

  // Trigger a rerender when the page number has changed. This needs to be done to update the
  // states of the arrows and page indicators.
  const handleScroll = useCallback(() => setPageNumberState(getPageNumber()), [getPageNumber]);

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <StyledCarousel $flexDirection="column" $gap="medium">
      <StyledPageSlider ref={pageContainerRef} onScroll={handleScroll}>
        {props.content.map((page, i) => (
          <StyledPage key={`page-${i}`}>{page}</StyledPage>
        ))}
      </StyledPageSlider>
      <CarouselControls
        goToPage={goToPage}
        hasNext={hasNext}
        hasPrev={hasPrev}
        next={next}
        prev={prev}
        pageNumber={pageNumber}
        numberOfPages={props.content.length}
      />
    </StyledCarousel>
  );
}
