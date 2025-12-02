import React from 'react';

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
    <IconButton ref={nextButtonRef} disabled={disabled} onClick={next} {...props}>
      <IconButton.Icon icon="chevron-right" />
    </IconButton>
  );
}
