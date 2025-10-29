import { useCarouselContext } from '../../CarouselContext';
import { CarouselControlGroup } from '../carousel-control-group';
import { CarouselIndicator } from '../carousel-indicator';

export function CarouselIndicators() {
  const { numberOfSlides, slideIndex } = useCarouselContext();
  return (
    <CarouselControlGroup>
      {[...Array(numberOfSlides)].map((_, i) => {
        const current = i === slideIndex;
        return <CarouselIndicator key={i} disabled={current} slideToGoTo={i} />;
      })}
    </CarouselControlGroup>
  );
}
