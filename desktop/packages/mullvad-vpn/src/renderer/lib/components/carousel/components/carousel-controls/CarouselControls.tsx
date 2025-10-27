import styled from 'styled-components';

import { Flex, IconButton, Layout } from '../../..';
import { PageIndicator } from '..';

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

export function CarouselControls(props: CarouselControlsProps) {
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
