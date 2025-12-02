import React from 'react';

import { IconButton, IconButtonProps } from '../../../icon-button';
import { useCarouselContext } from '../../CarouselContext';
import { useSlides } from '../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { prev, isFirstSlide } = useSlides();
  const { prevButtonRef } = useCarouselContext();
  const [disabled, setDisabled] = React.useState(isFirstSlide);

  // Allow focus to be moved before button is disabled.
  React.useEffect(() => {
    setDisabled(isFirstSlide);
  }, [isFirstSlide]);

  return (
    <IconButton ref={prevButtonRef} disabled={disabled} onClick={prev} {...props}>
      <IconButton.Icon icon="chevron-left" />
    </IconButton>
  );
}
