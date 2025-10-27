import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import { NonEmptyArray } from '../../../../shared/utils';
import { colors } from '../../foundations';
import { useStyledRef } from '../../utility-hooks';
import { Flex, IconButton, Layout } from '..';

const PAGE_GAP = 16;

const StyledPageSliderContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

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
    <StyledPageSliderContainer>
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
    </StyledPageSliderContainer>
  );
}

const StyledPageIndicator = styled.button`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: ${colors.whiteAlpha80};
  &&:hover {
    background-color: ${colors.whiteAlpha80};
  }
  &&:disabled {
    background-color: ${colors.whiteAlpha40};
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: '2px';
  }
`;

interface CarouselControlsProps {
  pageNumber: number;
  numberOfPages: number;
  hasNext: boolean;
  hasPrev: boolean;
  next: () => void;
  prev: () => void;
  goToPage: (page: number) => void;
}

const StyledGrid = styled(Layout)`
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
`;

function CarouselControls(props: CarouselControlsProps) {
  return (
    <StyledGrid $margin={{ top: 'small' }}>
      <div>{/* spacer to make page indicators centered */}</div>
      <Flex $gap="small">
        {[...Array(props.numberOfPages)].map((_, i) => {
          const current = i === props.pageNumber;
          return (
            <PageIndicator key={i} disabled={current} pageNumber={i} goToPage={props.goToPage} />
          );
        })}
      </Flex>
      <Flex $justifyContent="right" $gap="small">
        <IconButton disabled={!props.hasPrev} onClick={props.prev}>
          <IconButton.Icon icon="chevron-left" />
        </IconButton>
        <IconButton disabled={!props.hasNext} onClick={props.next}>
          <IconButton.Icon icon="chevron-right" />
        </IconButton>
      </Flex>
    </StyledGrid>
  );
}

type PageIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  pageNumber: number;
  goToPage: (page: number) => void;
};

function PageIndicator(props: PageIndicatorProps) {
  const { goToPage } = props;

  const onClick = useCallback(() => {
    goToPage(props.pageNumber);
  }, [goToPage, props.pageNumber]);

  return <StyledPageIndicator onClick={onClick} {...props} />;
}
