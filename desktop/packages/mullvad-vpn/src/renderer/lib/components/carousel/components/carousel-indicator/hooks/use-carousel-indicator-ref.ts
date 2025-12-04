import { useCarouselContext } from '../../../CarouselContext';

export function useCarouselIndicatorRef(slideToGoTo: number) {
  const { firstIndicatorRef, lastIndicatorRef } = useCarouselContext();
  const { numberOfSlides } = useCarouselContext();
  let ref = undefined;
  if (slideToGoTo === 0) {
    ref = firstIndicatorRef;
  } else if (slideToGoTo === numberOfSlides - 1) {
    ref = lastIndicatorRef;
  }
  return ref;
}
