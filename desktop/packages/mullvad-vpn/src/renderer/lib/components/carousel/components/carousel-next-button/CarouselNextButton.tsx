import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { IconButton, IconButtonProps } from '../../../icon-button';
import { useCarouselContext } from '../../CarouselContext';
import { useSlides } from '../../hooks';

export type CarouselNextButtonProps = IconButtonProps;

export function CarouselNextButton(props: CarouselNextButtonProps) {
  const { next, isLastSlide } = useSlides();
  const { nextButtonRef } = useCarouselContext();
  const [disabled, setDisabled] = React.useState(isLastSlide);

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabled(isLastSlide);
  }, [isLastSlide]);

  return (
    <IconButton
      ref={nextButtonRef}
      aria-label={
        // TRANSLATORS: Accessibility label for a button that navigates to the next slide in a carousel.
        messages.pgettext('accessibility', 'Next slide')
      }
      disabled={disabled}
      onClick={next}
      {...props}>
      <IconButton.Icon icon="chevron-right" />
    </IconButton>
  );
}
