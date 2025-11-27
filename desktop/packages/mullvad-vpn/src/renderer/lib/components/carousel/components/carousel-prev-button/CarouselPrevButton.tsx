import { Icon } from '../../../icon';
import { IconButton, IconButtonProps } from '../../../icon-button';
import { useSlides } from '../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { prev, isFirstSlide } = useSlides();

  const disabled = isFirstSlide;
  const tabIndex = disabled ? -1 : 0;

  return (
    <IconButton aria-disabled={disabled} tabIndex={tabIndex} onClick={prev} {...props}>
      {disabled ? (
        <Icon icon="chevron-left" color="whiteAlpha40" />
      ) : (
        <IconButton.Icon icon="chevron-left" />
      )}
    </IconButton>
  );
}
