import { IconButton, IconButtonProps } from '../../../icon-button';
import { useSlides } from '../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { prev, hasPrev } = useSlides();

  return (
    <IconButton disabled={!hasPrev} onClick={prev} {...props}>
      <IconButton.Icon icon="chevron-left" />
    </IconButton>
  );
}
