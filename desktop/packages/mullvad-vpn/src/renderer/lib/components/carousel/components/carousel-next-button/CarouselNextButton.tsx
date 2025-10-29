import { IconButton, IconButtonProps } from '../../../icon-button';
import { useSlides } from '../../hooks';

export type CarouselNextButtonProps = IconButtonProps;

export function CarouselNextButton(props: CarouselNextButtonProps) {
  const { next, hasNext } = useSlides();

  return (
    <IconButton disabled={!hasNext} onClick={next} {...props}>
      <IconButton.Icon icon="chevron-right" />
    </IconButton>
  );
}
