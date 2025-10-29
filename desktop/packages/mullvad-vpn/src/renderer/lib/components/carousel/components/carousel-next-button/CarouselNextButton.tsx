import { IconButton } from '../../../icon-button';
import { useSlides } from '../../hooks';

export function CarouselNextButton() {
  const { next, hasNext } = useSlides();

  return (
    <IconButton disabled={!hasNext} onClick={next}>
      <IconButton.Icon icon="chevron-right" />
    </IconButton>
  );
}
