import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';

type PageIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  pageNumber: number;
  goToPage: (page: number) => void;
};

const StyledPageIndicator = styled(Dot)`
  background-color: ${colors.whiteAlpha80};
`;

const StyledIconButton = styled.button`
  position: relative;
  display: flex;
  justify-content: center;
  border-radius: 50%;
  &&:hover ${StyledPageIndicator} {
    background-color: ${colors.whiteAlpha40};
  }
  &&:disabled ${StyledPageIndicator} {
    background-color: ${colors.whiteAlpha40};
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: 2px;
  }

  // Expand the clickable area
  &&::after {
    content: '';
    position: absolute;
    top: -4px;
    right: -4px;
    bottom: -4px;
    left: -4px;
  }
`;

export function PageIndicator(props: PageIndicatorProps) {
  const { goToPage } = props;

  const onClick = React.useCallback(() => {
    goToPage(props.pageNumber);
  }, [goToPage, props.pageNumber]);

  return (
    <StyledIconButton onClick={onClick} {...props}>
      <StyledPageIndicator size="tiny" />
    </StyledIconButton>
  );
}
