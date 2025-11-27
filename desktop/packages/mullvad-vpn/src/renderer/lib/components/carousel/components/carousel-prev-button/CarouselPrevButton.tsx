import { IconButton, IconButtonProps } from '../../../icon-button';
import { useSlides } from '../../hooks';

export type CarouselPrevButtonProps = IconButtonProps;

export function CarouselPrevButton(props: CarouselPrevButtonProps) {
  const { prev, isFirstSlide } = useSlides();

  return (
    <IconButton disabled={isFirstSlide} onClick={prev} {...props}>
      <IconButton.Icon icon="chevron-left" />
    </IconButton>
  );
}
