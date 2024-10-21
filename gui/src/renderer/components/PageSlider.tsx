import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { NonEmptyArray } from '../../shared/utils';
import { useStyledRef } from '../lib/utility-hooks';
import { Icon } from './cell';

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

export default function PageSlider(props: PageSliderProps) {
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
      <Controls
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

const StyledControlsContainer = styled.div({
  display: 'flex',
  marginTop: '12px',
  alignItems: 'center',
});

const StyledControlElement = styled.div({
  flex: '1 0 60px',
  display: 'flex',
});

const StyledArrows = styled(StyledControlElement)({
  display: 'flex',
  justifyContent: 'right',
  gap: '12px',
});

const StyledPageIndicators = styled(StyledControlElement)({
  display: 'flex',
  flexGrow: 2,
  justifyContent: 'center',
});

const StyledTransparentButton = styled.button({
  border: 'none',
  background: 'transparent',
  padding: '4px',
  margin: 0,
});

const StyledPageIndicator = styled.div<{ $current: boolean }>((props) => ({
  width: '8px',
  height: '8px',
  borderRadius: '50%',
  backgroundColor: props.$current ? colors.white80 : colors.white40,

  [`${StyledTransparentButton}:hover &&`]: {
    backgroundColor: colors.white80,
  },
}));

const StyledArrow = styled(Icon)((props) => ({
  backgroundColor: props.disabled ? colors.white20 : props.tintColor,

  [`${StyledTransparentButton}:hover &&`]: {
    backgroundColor: props.disabled ? colors.white20 : props.tintHoverColor,
  },
}));

const StyledLeftArrow = styled(StyledArrow)({
  transform: 'scaleX(-100%)',
});

interface ControlsProps {
  pageNumber: number;
  numberOfPages: number;
  hasNext: boolean;
  hasPrev: boolean;
  next: () => void;
  prev: () => void;
  goToPage: (page: number) => void;
}

function Controls(props: ControlsProps) {
  return (
    <StyledControlsContainer>
      <StyledControlElement>{/* spacer to make page indicators centered */}</StyledControlElement>
      <StyledPageIndicators>
        {[...Array(props.numberOfPages)].map((_, i) => (
          <PageIndicator
            key={i}
            current={i === props.pageNumber}
            pageNumber={i}
            goToPage={props.goToPage}
          />
        ))}
      </StyledPageIndicators>
      <StyledArrows>
        <StyledTransparentButton onClick={props.prev}>
          <StyledLeftArrow
            disabled={!props.hasPrev}
            height={12}
            width={7}
            source="icon-chevron"
            tintColor={colors.white}
            tintHoverColor={colors.white60}
          />
        </StyledTransparentButton>
        <StyledTransparentButton onClick={props.next}>
          <StyledArrow
            disabled={!props.hasNext}
            height={12}
            width={7}
            source="icon-chevron"
            tintColor={colors.white}
            tintHoverColor={colors.white60}
          />
        </StyledTransparentButton>
      </StyledArrows>
    </StyledControlsContainer>
  );
}

interface PageIndicatorProps {
  pageNumber: number;
  goToPage: (page: number) => void;
  current: boolean;
}

function PageIndicator(props: PageIndicatorProps) {
  const { goToPage } = props;

  const onClick = useCallback(() => {
    goToPage(props.pageNumber);
  }, [goToPage, props.pageNumber]);

  return (
    <StyledTransparentButton onClick={onClick}>
      <StyledPageIndicator $current={props.current} />
    </StyledTransparentButton>
  );
}
