import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { usePages } from '../../hooks';

export type PageIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  pageToGoTo: number;
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

export function PageIndicator({ pageToGoTo, ...props }: PageIndicatorProps) {
  const { goToPage } = usePages();

  const onClick = React.useCallback(() => {
    goToPage(pageToGoTo);
  }, [goToPage, pageToGoTo]);

  return (
    <StyledIconButton onClick={onClick} {...props}>
      <StyledPageIndicator size="tiny" />
    </StyledIconButton>
  );
}
