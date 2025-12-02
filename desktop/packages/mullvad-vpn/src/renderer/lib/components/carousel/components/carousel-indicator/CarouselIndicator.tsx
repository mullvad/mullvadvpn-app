import React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { useSlides } from '../../hooks';
import { useCarouselIndicatorRef } from './hooks';

export type CarouselIndicatorProps = React.ComponentPropsWithoutRef<'button'> & {
  slideToGoTo: number;
};

const StyledSlideIndicator = styled(Dot)`
  background-color: ${colors.white};
`;

const StyledCarouselIndicator = styled.button`
  --transition-duration: 0.15s;

  position: relative;
  display: flex;
  justify-content: center;
  border-radius: 50%;

  ${StyledSlideIndicator} {
    transition: background-color var(--transition-duration) ease;
  }

  &&:not(:disabled):hover ${StyledSlideIndicator} {
    --transition-duration: 0s;
    background-color: ${colors.whiteAlpha60};
  }

  &&:not(:disabled):active ${StyledSlideIndicator} {
    --transition-duration: 0s;
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

export function CarouselIndicator({
  slideToGoTo,
  disabled: disabledProp,
  ...props
}: CarouselIndicatorProps) {
  const { goToSlide } = useSlides();
  const ref = useCarouselIndicatorRef(slideToGoTo);

  const [disabled, setDisabled] = React.useState(disabledProp ?? false);

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabled(disabledProp ?? false);
  }, [disabledProp]);

  const handleClick = React.useCallback(() => {
    goToSlide(slideToGoTo);
  }, [goToSlide, slideToGoTo]);

  return (
    <StyledCarouselIndicator
      ref={ref}
      onClick={handleClick}
      disabled={disabled}
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for carousel indicators that allow navigation between slides.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(index)s - Will be replaced with the slide index number.
        messages.pgettext('accessibility', 'Go to slide %(index)s'),
        {
          index: slideToGoTo + 1,
        },
      )}
      {...props}>
      <StyledSlideIndicator size="tiny" />
    </StyledCarouselIndicator>
  );
}
