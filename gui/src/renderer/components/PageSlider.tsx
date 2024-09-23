import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { NonEmptyArray } from '../../shared/utils';
import { useStyledRef } from '../lib/utilityHooks';
import { Icon } from './cell';

// The amount of scroll required to switch page. This is compared with the `deltaX` value on the
// onWheel event.
const WHEEL_DELTA_THRESHOLD = 30;

const StyledPageSliderContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const StyledPageSlider = styled.div({
  whiteSpace: 'nowrap',
  overflow: 'hidden',
});

const StyledPage = styled.div({
  display: 'inline-block',
  width: '100%',
  whiteSpace: 'normal',
  verticalAlign: 'top',
});

interface PageSliderProps {
  content: NonEmptyArray<React.ReactNode>;
}

export default function PageSlider(props: PageSliderProps) {
  const [page, setPage] = useState(0);
  const pageContainerRef = useStyledRef<HTMLDivElement>();

  const hasNext = page < props.content.length - 1;
  const hasPrev = page > 0;

  const next = useCallback(() => {
    setPage((page) => Math.min(props.content.length - 1, page + 1));
  }, [props.content.length]);

  const prev = useCallback(() => {
    setPage((page) => Math.max(0, page - 1));
  }, []);

  // Go to next or previous page if the user scrolls horizontally.
  const onWheel = useCallback(
    (event: React.WheelEvent<HTMLDivElement>) => {
      if (event.deltaX > WHEEL_DELTA_THRESHOLD) {
        next();
      } else if (event.deltaX < -WHEEL_DELTA_THRESHOLD) {
        prev();
      }
    },
    [next, prev],
  );

  // Scroll to the correct position when the page prop changes.
  useEffect(() => {
    if (pageContainerRef.current) {
      // The page width is the same as the container width.
      const width = pageContainerRef.current.offsetWidth;
      pageContainerRef.current.scrollTo({ left: width * page, behavior: 'smooth' });
    }
  }, [page]);

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

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <StyledPageSliderContainer>
      <StyledPageSlider ref={pageContainerRef} onWheel={onWheel}>
        {props.content.map((page, i) => (
          <StyledPage key={`page-${i}`}>{page}</StyledPage>
        ))}
      </StyledPageSlider>
      <Controls
        goToPage={setPage}
        hasNext={hasNext}
        hasPrev={hasPrev}
        next={next}
        prev={prev}
        page={page}
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
  page: number;
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
          <PageIndicator key={i} current={i === props.page} page={i} goToPage={props.goToPage} />
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
  page: number;
  goToPage: (page: number) => void;
  current: boolean;
}

function PageIndicator(props: PageIndicatorProps) {
  const onClick = useCallback(() => {
    props.goToPage(props.page);
  }, [props.goToPage, props.page]);

  return (
    <StyledTransparentButton onClick={onClick}>
      <StyledPageIndicator $current={props.current} />
    </StyledTransparentButton>
  );
}
