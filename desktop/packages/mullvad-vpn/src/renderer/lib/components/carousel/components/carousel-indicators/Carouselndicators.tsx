import { createRange } from '../../../../utils';
import { useCarouselContext } from '../../CarouselContext';
import { CarouselControlGroup } from '../carousel-control-group';
import { CarouselIndicator } from '../carousel-indicator';

export function CarouselIndicators() {
  const { numberOfSlides, slideIndex } = useCarouselContext();

  const range = createRange(numberOfSlides);
  return (
    <CarouselControlGroup>
      {range.map((_, i) => {
        const current = i === slideIndex;
        return <CarouselIndicator key={i} disabled={current} slideToGoTo={i} />;
      })}
    </CarouselControlGroup>
  );
}
