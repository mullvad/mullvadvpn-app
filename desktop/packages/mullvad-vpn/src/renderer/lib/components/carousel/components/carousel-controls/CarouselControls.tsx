import styled from 'styled-components';

import { Flex, IconButton } from '../../..';
import { useCarouselContext } from '../../CarouselContext';
import { useSlides } from '../../hooks';
import { SlideIndicator } from '..';

const StyledGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
`;

export function CarouselControls() {
  const { numberOfSlides, slideIndex } = useCarouselContext();
  const { next, prev, hasNext, hasPrev } = useSlides();
  return (
    <StyledGrid>
      <div>{/* spacer to make slide indicators centered */}</div>
      <Flex $gap="small">
        {[...Array(numberOfSlides)].map((_, i) => {
          const current = i === slideIndex;
          return <SlideIndicator key={i} disabled={current} slideToGoTo={i} />;
        })}
      </Flex>
      <Flex $justifyContent="right" $gap="small">
        <IconButton disabled={!hasPrev} onClick={prev}>
          <IconButton.Icon icon="chevron-left" />
        </IconButton>
        <IconButton disabled={!hasNext} onClick={next}>
          <IconButton.Icon icon="chevron-right" />
        </IconButton>
      </Flex>
    </StyledGrid>
  );
}
