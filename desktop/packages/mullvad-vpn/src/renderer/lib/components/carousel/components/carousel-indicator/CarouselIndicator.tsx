import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { useSlides } from '../../hooks';

export type CarouselIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  slideToGoTo: number;
};

const StyledSlideIndicator = styled(Dot)`
  background-color: ${colors.whiteAlpha80};
`;

const StyledCarouselIndicator = styled.button`
  position: relative;
  display: flex;
  justify-content: center;
  border-radius: 50%;
  &&:hover ${StyledSlideIndicator} {
    background-color: ${colors.whiteAlpha40};
  }
  &&:disabled ${StyledSlideIndicator} {
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

export function CarouselIndicator({ slideToGoTo, ...props }: CarouselIndicatorProps) {
  const { goToSlide } = useSlides();

  const onClick = React.useCallback(() => {
    goToSlide(slideToGoTo);
  }, [goToSlide, slideToGoTo]);

  return (
    <StyledCarouselIndicator onClick={onClick} {...props}>
      <StyledSlideIndicator size="tiny" />
    </StyledCarouselIndicator>
  );
}
