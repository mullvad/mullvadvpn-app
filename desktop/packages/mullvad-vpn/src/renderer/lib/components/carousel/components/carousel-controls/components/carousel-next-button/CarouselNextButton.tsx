import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useEffectEvent } from '../../../../../../utility-hooks';
import { IconButton, IconButtonProps } from '../../../../../icon-button';
import { useCarouselContext } from '../../../../CarouselContext';
import { useSlides } from '../../../../hooks';

export type CarouselNextButtonProps = IconButtonProps;

export function CarouselNextButton(props: CarouselNextButtonProps) {
  const { goToNextSlide, isLastSlide } = useSlides();
  const { nextButtonRef } = useCarouselContext();
  const [disabled, setDisabled] = React.useState(isLastSlide);

  // TODO: Remove the use of useEffectEvent. This is used as an escape hatch
  // in order to be able to continue setting state from a useEffect without
  // lint errors.
  //
  // The entire logic should be rewritten to no longer depend on setting
  // state from an effect.
  const setDisabledEffectEvent = useEffectEvent((value: boolean) => {
    setDisabled(value);
  });

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabledEffectEvent(isLastSlide);
  }, [isLastSlide]);

  return (
    <IconButton
      ref={nextButtonRef}
      aria-label={
        // TRANSLATORS: Accessibility label for a button that navigates to the next slide in a carousel.
        messages.pgettext('accessibility', 'Next slide')
      }
      disabled={disabled}
      onClick={goToNextSlide}
      {...props}>
      <IconButton.Icon icon="chevron-right" />
    </IconButton>
  );
}
