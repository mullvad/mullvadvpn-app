import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useEffectEvent } from '../../../../../../utility-hooks';
import { IconButton, IconButtonProps } from '../../../../../icon-button';
import { useCarouselContext } from '../../../../CarouselContext';
import { useSlides } from '../../../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { goToPreviousSlide, isFirstSlide } = useSlides();
  const { prevButtonRef } = useCarouselContext();
  const [disabled, setDisabled] = React.useState(isFirstSlide);

  // TODO: Remove the use of useEffectEvent. This is used as an escape hatch
  // in order to be able to continue setting state from a useEffect without
  // lint errors.
  //
  // The entire logic should be rewritten to no longer depend on setting
  // state from settings state in an effect.
  const setDisabledEffectEvent = useEffectEvent((value: boolean) => {
    setDisabled(value);
  });

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabledEffectEvent(isFirstSlide);
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
