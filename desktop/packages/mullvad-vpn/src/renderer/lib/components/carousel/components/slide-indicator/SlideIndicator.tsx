import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { useSlides } from '../../hooks';

export type PageIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  slideToGoTo: number;
};

const StyledPageIndicator = styled(Dot)`
  background-color: ${colors.whiteAlpha80};
`;

const StyledSlideIndicator = styled.button`
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

export function SlideIndicator({ slideToGoTo: pageToGoTo, ...props }: PageIndicatorProps) {
  const { goToSlide } = useSlides();

  const onClick = React.useCallback(() => {
    goToSlide(pageToGoTo);
  }, [goToSlide, pageToGoTo]);

  return (
    <StyledSlideIndicator onClick={onClick} {...props}>
      <StyledPageIndicator size="tiny" />
    </StyledSlideIndicator>
  );
}
