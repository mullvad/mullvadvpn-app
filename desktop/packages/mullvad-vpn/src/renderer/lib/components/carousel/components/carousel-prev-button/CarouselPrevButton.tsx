import { IconButton } from '../../../icon-button';
import { useSlides } from '../../hooks';

export function CarouselPrevButton() {
  const { prev, hasPrev } = useSlides();

  return (
    <IconButton disabled={!hasPrev} onClick={prev}>
      <IconButton.Icon icon="chevron-left" />
    </IconButton>
  );
}
