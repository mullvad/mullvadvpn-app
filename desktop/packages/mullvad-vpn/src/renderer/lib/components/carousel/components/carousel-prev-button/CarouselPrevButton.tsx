import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { IconButton, IconButtonProps } from '../../../icon-button';
import { useCarouselContext } from '../../CarouselContext';
import { useSlides } from '../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { goToPreviousSlide, isFirstSlide } = useSlides();
  const { prevButtonRef } = useCarouselContext();
  const [disabled, setDisabled] = React.useState(isFirstSlide);

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabled(isFirstSlide);
  }, [isFirstSlide]);

  return (
    <IconButton
      ref={prevButtonRef}
      aria-label={
        // TRANSLATORS: Accessibility label for a button that navigates to the previous slide in a carousel.
        messages.pgettext('accessibility', 'Previous slide')
      }
      disabled={disabled}
      onClick={goToPreviousSlide}
      {...props}>
      <IconButton.Icon icon="chevron-left" />
    </IconButton>
  );
}
