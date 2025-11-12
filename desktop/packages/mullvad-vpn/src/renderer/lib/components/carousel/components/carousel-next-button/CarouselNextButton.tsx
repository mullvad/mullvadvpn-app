import { Icon } from '../../../icon';
import { IconButton, IconButtonProps } from '../../../icon-button';
import { useSlides } from '../../hooks';

export type CarouselNextButtonProps = IconButtonProps;

export function CarouselNextButton(props: CarouselNextButtonProps) {
  const { next, hasNext } = useSlides();

  const disabled = !hasNext;

  return (
    <IconButton aria-disabled={disabled} tabIndex={disabled ? -1 : 0} onClick={next} {...props}>
      {disabled ? (
        <Icon icon="chevron-right" color="whiteAlpha40" />
      ) : (
        <IconButton.Icon icon="chevron-right" />
      )}
    </IconButton>
  );
}
