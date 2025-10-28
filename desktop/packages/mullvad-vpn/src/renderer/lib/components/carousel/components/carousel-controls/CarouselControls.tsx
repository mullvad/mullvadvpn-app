import styled from 'styled-components';

import { Flex, IconButton } from '../../..';
import { useCarouselContext } from '../../CarouselContext';
import { usePages } from '../../hooks';
import { PageIndicator } from '..';

const StyledGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
`;

export function CarouselControls() {
  const { numberOfPages, pageNumber } = useCarouselContext();
  const { next, prev, hasNext, hasPrev } = usePages();
  return (
    <StyledGrid>
      <div>{/* spacer to make page indicators centered */}</div>
      <Flex $gap="small">
        {[...Array(numberOfPages)].map((_, i) => {
          const current = i === pageNumber;
          return <PageIndicator key={i} disabled={current} pageToGoTo={i} />;
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
